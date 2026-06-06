use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::adapters::cache::{CacheManager, cache_key};
use crate::adapters::structured_logging::log_business_event;
use crate::adapters::telemetry::Telemetry;
use crate::domain::types::GraphStats;
use crate::server::mcp_handler::EpistemeMCP;

use crate::server::api_models::{
    AnalyzeRequest, AnalyzeResponse, Components, ErrorResponse, GraphNeighborsRequest,
    GraphPathRequest, GraphPathResponse, HealthResponse, RefactorRequest, RefactorResponse,
    SearchRequest, SearchResponse, StatsResponse, SubgraphRequest, SystemInfo,
};

/// Map an MCP error JSON value to an appropriate HTTP error response.
fn mcp_error_response(
    result: &serde_json::Value,
    fallback_msg: &str,
) -> Option<axum::response::Response> {
    let err = result.get("error")?;
    let msg = err.as_str().unwrap_or(fallback_msg);
    let status = if msg.contains("exceeds") {
        StatusCode::PAYLOAD_TOO_LARGE
    } else if msg.contains("Unsupported") {
        StatusCode::BAD_REQUEST
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };
    Some((status, Json(ErrorResponse { error: msg.into() })).into_response())
}

/// Shared application state passed to all handlers.
pub type AppState = Arc<EpistemeMCP>;

// ---------------------------------------------------------------------------
// Query parameter structs for GET endpoints
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateInsightRequest {
    pub text: String,
    pub tags: Option<Vec<String>>,
    pub linked_entities: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<usize>,
    #[serde(rename = "type")]
    pub entity_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NeighborsQuery {
    #[serde(rename = "type")]
    pub relation_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EntityQuery {
    pub detail: Option<String>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

fn gather_components(mcp: &EpistemeMCP) -> Components {
    let knowledge_graph = if mcp.graph().all_entity_ids().is_empty() {
        "empty".to_owned()
    } else {
        "loaded".to_owned()
    };
    let rag_database = if mcp.has_db() {
        "connected".to_owned()
    } else {
        "not_available".to_owned()
    };
    let embedding_provider = if mcp.has_embedding_provider() {
        "configured".to_owned()
    } else {
        "noop".to_owned()
    };

    let (embedding_model, embedding_dim) = mcp
        .embedding_info()
        .map(|info| (info.stored_model.clone(), info.stored_dim))
        .unwrap_or((None, None));

    Components {
        knowledge_graph,
        rag_database,
        embedding_provider,
        embedding_model,
        embedding_dim,
    }
}

fn gather_system_info() -> SystemInfo {
    use sysinfo::System;

    let mut sys = System::new();
    sys.refresh_memory();
    sys.refresh_cpu_usage();

    let memory_used_mb = sys.used_memory() as f64 / 1024.0 / 1024.0;
    let memory_total_mb = sys.total_memory() as f64 / 1024.0 / 1024.0;

    let disks = sysinfo::Disks::new_with_refreshed_list();
    let (disk_used_gb, disk_total_gb) = disks.first().map_or((0.0, 0.0), |d| {
        (
            (d.total_space() - d.available_space()) as f64 / 1024.0 / 1024.0 / 1024.0,
            d.total_space() as f64 / 1024.0 / 1024.0 / 1024.0,
        )
    });

    SystemInfo {
        memory_used_mb,
        memory_total_mb,
        cpu_usage_percent: sys.global_cpu_usage(),
        disk_used_gb,
        disk_total_gb,
    }
}

pub async fn health(
    State(mcp): State<AppState>,
    Extension(cache): Extension<Arc<CacheManager>>,
) -> impl IntoResponse {
    let components = gather_components(&mcp);
    let system = gather_system_info();
    let uptime = crate::server::api_app::START_TIME.elapsed().as_secs();
    let cache_stats = cache.get_stats().await;

    Json(HealthResponse {
        status: "ok".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        uptime_secs: uptime,
        components: Some(components),
        system: Some(system),
        cache: Some(crate::server::api_models::CacheInfo {
            enabled: cache.is_enabled(),
            connected: cache.is_connected().await,
            keys: cache_stats.keys,
            total_keys: cache_stats.total_keys,
            hits: cache_stats.hits,
            misses: cache_stats.misses,
            hit_rate: cache_stats.hit_rate,
        }),
    })
}

pub async fn stats(State(mcp): State<AppState>) -> impl IntoResponse {
    let resource = mcp.handle_resource_read("episteme://stats");
    let embedding = mcp.embedding_info();
    match serde_json::from_value::<GraphStats>(resource.clone()) {
        Ok(gs) => Json(StatsResponse {
            total_entities: gs.total_entities,
            total_edges: gs.total_edges,
            by_type: gs.by_type,
            embedding,
        }),
        Err(_) => {
            // Fallback: if the resource returned something unexpected, try
            // constructing a minimal response from the raw JSON.
            let total_entities = resource
                .get("total_entities")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize;
            let total_edges = resource
                .get("total_edges")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize;
            let by_type = resource
                .get("by_type")
                .and_then(|v| v.as_object())
                .map(|obj| {
                    obj.iter()
                        .filter_map(|(k, v)| v.as_u64().map(|n| (k.clone(), n as usize)))
                        .collect()
                })
                .unwrap_or_default();
            Json(StatsResponse {
                total_entities,
                total_edges,
                by_type,
                embedding,
            })
        }
    }
}

pub async fn analyze(
    State(mcp): State<AppState>,
    Extension(cache): Extension<Arc<CacheManager>>,
    Extension(telemetry): Extension<Arc<Telemetry>>,
    Json(body): Json<AnalyzeRequest>,
) -> impl IntoResponse {
    let _timer = crate::adapters::metrics::AnalysisTimer::new();
    let min_confidence = body.min_confidence.unwrap_or(0.5).clamp(0.0, 1.0);

    // Check cache before computing.
    let lang_label = body.language.as_deref().unwrap_or("");
    let min_conf_label = format!("{min_confidence:.3}");
    let ck = cache_key("analyze", &[&body.code, lang_label, &min_conf_label]);
    if let Some(cached) = cache.get(&ck).await
        && let Some(smells) = cached.get("smells").and_then(|v| v.as_array()).cloned()
    {
        let count = smells.len();
        return (StatusCode::OK, Json(AnalyzeResponse { smells, count })).into_response();
    }

    let result = mcp.analyze_code(&body.code, body.language.as_deref());

    if let Some(resp) = mcp_error_response(&result, "analysis error") {
        return resp;
    }

    let smells: Vec<serde_json::Value> = result
        .get("smells")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|smell| {
            smell
                .get("confidence")
                .and_then(|v| v.as_f64())
                .map(|c| c >= min_confidence)
                .unwrap_or(true)
        })
        .collect();
    let count = smells.len();

    // Track each detected smell.
    for smell in &smells {
        let smell_id = smell
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let smell_name = smell
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        crate::adapters::metrics::track_smell_detection(smell_id, smell_name);
    }

    // Store successful result in cache.
    let response_value = serde_json::to_value(&AnalyzeResponse {
        smells: smells.clone(),
        count,
    })
    .unwrap_or(serde_json::Value::Null);
    cache.set(&ck, &response_value, None).await;
    log_business_event(
        "analyze_completed",
        serde_json::json!({"count": count, "language": body.language}),
    );
    telemetry.track_event(
        "analyze_completed",
        serde_json::json!({"count": count, "language": body.language}),
    );

    (StatusCode::OK, Json(AnalyzeResponse { smells, count })).into_response()
}

pub async fn refactor(
    State(mcp): State<AppState>,
    Extension(telemetry): Extension<Arc<Telemetry>>,
    Json(body): Json<RefactorRequest>,
) -> impl IntoResponse {
    let result = mcp.suggest_refactorings(&body.code, body.language.as_deref(), body.top_k);
    let min_confidence = body.min_confidence.unwrap_or(0.5).clamp(0.0, 1.0);

    if let Some(resp) = mcp_error_response(&result, "refactoring error") {
        return resp;
    }

    let analyses: Vec<serde_json::Value> = result
        .get("analyses")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|analysis| {
            analysis
                .get("smell")
                .and_then(|v| v.get("confidence"))
                .and_then(|v| v.as_f64())
                .map(|c| c >= min_confidence)
                .unwrap_or(true)
        })
        .collect();
    let count = analyses.len();

    // Track each refactoring suggestion.
    for analysis in &analyses {
        if let Some(suggestions) = analysis.get("suggestions").and_then(|v| v.as_array()) {
            for suggestion in suggestions {
                let refactoring_id = suggestion
                    .get("refactoring_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                crate::adapters::metrics::track_refactoring_suggestion(refactoring_id);
            }
        }
    }
    telemetry.track_event(
        "refactor_completed",
        serde_json::json!({"count": count, "language": body.language}),
    );

    (StatusCode::OK, Json(RefactorResponse { analyses, count })).into_response()
}

/// Shared search logic used by both `GET /search` and `POST /search`.
async fn do_search(
    mcp: &AppState,
    cache: &CacheManager,
    telemetry: &Telemetry,
    query: &str,
    limit: Option<usize>,
    entity_type: Option<&str>,
) -> SearchResponse {
    let _timer = crate::adapters::metrics::SearchTimer::new();

    // Check cache before computing.
    let limit_label = limit.map(|l| l.to_string()).unwrap_or_default();
    let type_label = entity_type.unwrap_or("");
    let ck = cache_key("search", &[query, &limit_label, type_label]);
    if let Some(cached) = cache.get(&ck).await
        && let Some(results) = cached.get("results").and_then(|v| v.as_array()).cloned()
    {
        let count = results.len();
        return SearchResponse { results, count };
    }

    let result = mcp.search_knowledge(query, limit, entity_type);

    let results = result
        .get("results")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let count = results.len();

    let entity_type_label = entity_type.unwrap_or("any");
    let has_filter = entity_type.is_some();
    crate::adapters::metrics::track_search(entity_type_label, has_filter);

    // Store successful result in cache.
    let response_value = serde_json::to_value(&SearchResponse {
        results: results.clone(),
        count,
    })
    .unwrap_or(serde_json::Value::Null);
    cache.set(&ck, &response_value, None).await;
    telemetry.track_event(
        "search_completed",
        serde_json::json!({"count": count, "entity_type": entity_type}),
    );

    SearchResponse { results, count }
}

pub async fn search(
    State(mcp): State<AppState>,
    Extension(cache): Extension<Arc<CacheManager>>,
    Extension(telemetry): Extension<Arc<Telemetry>>,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    Json(
        do_search(
            &mcp,
            &cache,
            &telemetry,
            &params.q,
            params.limit,
            params.entity_type.as_deref(),
        )
        .await,
    )
}

/// Python-parity endpoint: `POST /search` with JSON body.
pub async fn search_post(
    State(mcp): State<AppState>,
    Extension(cache): Extension<Arc<CacheManager>>,
    Extension(telemetry): Extension<Arc<Telemetry>>,
    Json(body): Json<SearchRequest>,
) -> impl IntoResponse {
    Json(
        do_search(
            &mcp,
            &cache,
            &telemetry,
            &body.query,
            body.limit,
            body.entity_type.as_deref(),
        )
        .await,
    )
}

pub async fn get_entity(
    State(mcp): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<EntityQuery>,
) -> impl IntoResponse {
    let result = mcp.get_entity(&id, params.detail.as_deref());

    if let Some(err) = result.get("error") {
        let msg = err.as_str().unwrap_or("entity not found");
        return (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse { error: msg.into() }),
        )
            .into_response();
    }

    Json(result).into_response()
}

pub async fn get_neighbors(
    State(mcp): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<NeighborsQuery>,
) -> impl IntoResponse {
    let result = mcp.get_neighbors(&id, params.relation_type.as_deref());

    if let Some(err) = result.get("error") {
        let msg = err.as_str().unwrap_or("entity not found");
        return (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse { error: msg.into() }),
        )
            .into_response();
    }

    Json(result).into_response()
}

/// Python-parity endpoint: `POST /graph/neighbors`.
pub async fn get_neighbors_post(
    State(mcp): State<AppState>,
    Json(body): Json<GraphNeighborsRequest>,
) -> impl IntoResponse {
    let result = mcp.get_neighbors(&body.entity_id, body.relation_type.as_deref());
    if let Some(err) = result.get("error") {
        let msg = err.as_str().unwrap_or("entity not found");
        return (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse { error: msg.into() }),
        )
            .into_response();
    }
    Json(result).into_response()
}

pub async fn graph_path(
    State(mcp): State<AppState>,
    Json(body): Json<GraphPathRequest>,
) -> impl IntoResponse {
    let result = mcp.find_path(&body.from_id, &body.to_id, body.max_depth);

    if let Some(err) = result.get("error") {
        let msg = err.as_str().unwrap_or("no path found");
        return (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse { error: msg.into() }),
        )
            .into_response();
    }

    let from = result
        .get("from")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_owned();
    let to = result
        .get("to")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_owned();
    let length = result.get("length").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let path = result
        .get("path")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    Json(GraphPathResponse {
        from,
        to,
        length,
        path,
    })
    .into_response()
}

pub async fn contradictions(State(mcp): State<AppState>) -> impl IntoResponse {
    let result = mcp.handle_resource_read("episteme://contradictions");
    Json(result)
}

// ---------------------------------------------------------------------------
// Root info
// ---------------------------------------------------------------------------

pub async fn root_info() -> impl IntoResponse {
    Json(serde_json::json!({
        "name": "episteme",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Software engineering knowledge graph API",
        "endpoints": [
            "/health",
            "/health/live",
            "/health/ready",
            "/api/stats",
            "/api/analyze",
            "/api/refactor",
            "/api/search",
            "/api/entity/{id}",
            "/api/entity/{id}/neighbors",
            "/api/graph/path",
            "/api/graph/subgraph",
            "/api/graph/infer",
            "/api/contradictions",
            "/mcp",
        ],
    }))
}

// ---------------------------------------------------------------------------
// Kubernetes health probes
// ---------------------------------------------------------------------------

pub async fn liveness() -> impl IntoResponse {
    Json(serde_json::json!({"status": "alive"}))
}

pub async fn readiness(State(mcp): State<AppState>) -> impl IntoResponse {
    let stats = mcp.graph().stats();
    if stats.total_entities > 0 {
        (
            StatusCode::OK,
            Json(serde_json::json!({"status": "ready", "entities": stats.total_entities})),
        )
            .into_response()
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"status": "not_ready", "entities": 0})),
        )
            .into_response()
    }
}

// ---------------------------------------------------------------------------
// Subgraph extraction
// ---------------------------------------------------------------------------

pub async fn subgraph(
    State(mcp): State<AppState>,
    Json(body): Json<SubgraphRequest>,
) -> impl IntoResponse {
    let depth = body.depth.unwrap_or(2);
    let graph = mcp.graph();

    match graph.get_entity(&body.entity_id) {
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Entity '{}' not found", body.entity_id),
            }),
        )
            .into_response(),
        Some(_) => {
            let subgraph = graph.extract_subgraph(&body.entity_id, depth);
            Json(serde_json::to_value(subgraph).unwrap_or(serde_json::Value::Null)).into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// Transitive relation inference
// ---------------------------------------------------------------------------

pub async fn infer_transitive(State(mcp): State<AppState>) -> impl IntoResponse {
    let graph = mcp.graph();
    let inferred = graph.infer_transitive_enforcements();
    Json(serde_json::json!({
        "count": inferred.len(),
        "inferred_enforcements": inferred,
    }))
}

// ---------------------------------------------------------------------------
// Debug / profiling
// ---------------------------------------------------------------------------

pub async fn debug_profile() -> Json<serde_json::Value> {
    use sysinfo::System;

    let mut sys = System::new();
    sys.refresh_all();

    let pid = sysinfo::get_current_pid().unwrap_or(sysinfo::Pid::from(0));
    let process = sys.process(pid);

    let process_info = if let Some(p) = process {
        serde_json::json!({
            "pid": pid.as_u32(),
            "memory_mb": p.memory() as f64 / 1024.0,
            "cpu_usage": p.cpu_usage(),
            "start_time": p.start_time(),
        })
    } else {
        serde_json::json!({"pid": pid.as_u32(), "error": "process not found"})
    };

    Json(serde_json::json!({
        "process": process_info,
        "system": {
            "total_memory_mb": sys.total_memory() / 1024,
            "used_memory_mb": sys.used_memory() / 1024,
            "cpu_usage": sys.global_cpu_usage(),
            "uptime_secs": System::uptime(),
        },
        "build": {
            "version": env!("CARGO_PKG_VERSION"),
            "debug_assertions": cfg!(debug_assertions),
        },
    }))
}

/// POST /insights -- create a tacit knowledge insight.
pub async fn create_insight(
    State(mcp): State<AppState>,
    Json(body): Json<CreateInsightRequest>,
) -> impl IntoResponse {
    if body.text.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "text must not be empty"})),
        );
    }

    let result = mcp.add_insight(&body.text, body.tags, body.linked_entities, None);

    if result.get("error").is_some() {
        (StatusCode::UNPROCESSABLE_ENTITY, Json(result))
    } else {
        (StatusCode::CREATED, Json(result))
    }
}
