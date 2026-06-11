use std::sync::Arc;
use std::time::Instant;

use crate::domain::graph::KnowledgeGraph;
use crate::server::mcp_handler::EpistemeMCP;
use axum::{Extension, Router, middleware as axum_mw, routing::get, routing::post};
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::trace::TraceLayer;

use crate::adapters::cache::CacheManager;
use crate::adapters::metrics::{self, MetricsHandle};
use crate::adapters::rate_limiter::RateLimiter;
use crate::adapters::rate_limiter_mw::rate_limit_middleware;
use crate::adapters::telemetry::Telemetry;
use crate::adapters::user_graph_store::UserGraphStore;
use crate::domain::composite_graph::CompositeGraph;
use crate::server::api_middleware::{
    ApiKeys, auth_middleware, cors_layer, global_error_handler, request_id_middleware,
};
use crate::server::api_routes::AppState;
use crate::server::mcp_transport_http;

/// Server start time, used for uptime reporting.
pub static START_TIME: std::sync::LazyLock<Instant> = std::sync::LazyLock::new(Instant::now);

/// Build the complete axum `Router` with all routes and middleware.
///
/// Route structure matches the Python FastAPI layout exactly:
/// `/health`, `/health/live`, `/health/ready`, `/stats`, `/analyze`, `/refactor`,
/// `/search`, `/graph/{id}`, `/graph/{id}/neighbors`, `/graph/path`,
/// `/graph/subgraph`, `/graph/contradictions`, `/graph/infer`.
///
/// MCP convenience endpoints:
/// `/mcp` (POST), `/tools` (GET), `/resources` (GET), `/tool` (POST).
///
/// Observability:
/// `/metrics` (GET) serves Prometheus-formatted metrics.
///
/// `api_keys` is a parsed list of allowed keys. When empty, auth is skipped entirely.
/// Auth middleware is applied to POST endpoints but skipped for GET /health and GET /tools.
///
/// `redis_host`, `redis_port`, `redis_db`, `redis_ttl`, `redis_enabled` configure
/// the optional Redis response cache.
///
/// `enable_debug` controls whether debug/profile endpoints are mounted.
#[allow(clippy::too_many_arguments)]
pub async fn create_app(
    graph: KnowledgeGraph,
    api_keys: Vec<String>,
    cors_origins: &str,
    redis_host: &str,
    redis_port: u16,
    redis_db: u16,
    redis_ttl: u64,
    redis_enabled: bool,
    enable_debug: bool,
    telemetry_enabled: bool,
    posthog_api_key: String,
    posthog_host: String,
    sentry_dsn: String,
) -> Router {
    // Initialise Prometheus metrics recorder once.
    let metrics_handle = metrics::init_metrics();

    // Default rate limiter: 100 req/min for general routes. Per-route limits
    // are applied dynamically inside the middleware based on the path.
    let limiter = Arc::new(RateLimiter::new(100));

    // Set up the Redis cache manager.
    let cache = Arc::new(CacheManager::new(redis_enabled, redis_ttl));
    if let Err(e) = cache
        .connect(redis_host, redis_port, u64::from(redis_db))
        .await
    {
        tracing::warn!("Redis cache connection failed, caching disabled: {e}");
    }
    let telemetry = Arc::new(Telemetry::new(
        telemetry_enabled,
        posthog_api_key,
        posthog_host,
        sentry_dsn,
    ));

    let db_path = crate::adapters::paths::episteme_home().join("user_knowledge.db");
    let mut mcp = match UserGraphStore::open(&db_path) {
        Ok(store) => {
            let composite = CompositeGraph::new(graph.clone(), Box::new(store));
            EpistemeMCP::with_composite(graph, composite)
        }
        Err(_) => EpistemeMCP::new(graph),
    };
    mcp.try_attach_rag();
    let state: AppState = Arc::new(mcp);
    let api_keys_state = Arc::new(ApiKeys(api_keys));

    let mut router = Router::new()
        // Root info
        .route("/", get(crate::server::api_routes::root_info))
        // Health probes
        .route("/health", get(crate::server::api_routes::health))
        .route("/health/live", get(crate::server::api_routes::liveness))
        .route("/health/ready", get(crate::server::api_routes::readiness))
        // MCP JSON-RPC over HTTP (POST only)
        .route("/mcp", post(mcp_transport_http::handle_mcp_post))
        // MCP convenience GET endpoints (no auth required)
        .route("/tools", get(mcp_transport_http::handle_tools_list))
        .route("/resources", get(mcp_transport_http::handle_resources_list))
        // MCP convenience POST endpoint (auth required)
        .route("/tool", post(mcp_transport_http::handle_tool_call))
        // Stats
        .route("/stats", get(crate::server::api_routes::stats))
        // Code analysis
        .route("/analyze", post(crate::server::api_routes::analyze))
        .route("/refactor", post(crate::server::api_routes::refactor))
        // Search
        .route("/search", get(crate::server::api_routes::search))
        .route("/search", post(crate::server::api_routes::search_post))
        // Graph queries
        .route("/graph/{id}", get(crate::server::api_routes::get_entity))
        .route(
            "/graph/{id}/neighbors",
            get(crate::server::api_routes::get_neighbors),
        )
        .route(
            "/graph/neighbors",
            post(crate::server::api_routes::get_neighbors_post),
        )
        .route("/graph/path", post(crate::server::api_routes::graph_path))
        .route("/graph/subgraph", post(crate::server::api_routes::subgraph))
        .route(
            "/graph/contradictions",
            get(crate::server::api_routes::contradictions),
        )
        .route(
            "/graph/infer",
            get(crate::server::api_routes::infer_transitive),
        )
        // Insight creation
        .route("/insights", post(crate::server::api_routes::create_insight))
        .route("/insights", get(crate::server::api_routes::list_insights))
        // Prometheus metrics endpoint
        .route("/metrics", get(metrics_endpoint));

    // Debug / profiling endpoints (gated behind config flag)
    if enable_debug {
        router = router.route(
            "/debug/profile",
            get(crate::server::api_routes::debug_profile),
        );
    }

    router
        // Middleware (outermost first, so request-id runs before auth)
        .layer(CatchPanicLayer::custom(global_error_handler))
        .layer(axum_mw::from_fn(request_id_middleware))
        .layer(axum_mw::from_fn_with_state(limiter, rate_limit_middleware))
        .layer(axum_mw::from_fn_with_state(api_keys_state, auth_middleware))
        .layer(Extension(metrics_handle))
        .layer(Extension(cache))
        .layer(Extension(telemetry))
        .layer(TraceLayer::new_for_http())
        .layer(cors_layer(cors_origins))
        .with_state(state)
}

/// Handler for `GET /metrics`. Returns Prometheus text-format metrics.
async fn metrics_endpoint(
    Extension(handle): Extension<Arc<MetricsHandle>>,
) -> impl axum::response::IntoResponse {
    (
        [(
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_static("text/plain; version=0.0.4; charset=utf-8"),
        )],
        handle.render(),
    )
}
