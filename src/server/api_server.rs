use std::net::SocketAddr;

use anyhow::Result;
use crate::adapters::config::SyntagmaConfig;
use crate::domain::graph::KnowledgeGraph;

use crate::server::api_app::create_app;
use crate::server::mcp_auth::parse_api_keys;

/// Start the REST API server with graceful shutdown.
///
/// Binds to `config.api_host:config.api_port` and serves the axum application.
/// Shuts down cleanly on ctrl-c (SIGINT).
pub async fn run(config: &SyntagmaConfig, graph: KnowledgeGraph) -> Result<()> {
    let addr: SocketAddr = format!("{}:{}", config.api_host, config.api_port)
        .parse()
        .map_err(|e| anyhow::anyhow!("invalid bind address: {e}"))?;

    let api_keys = parse_api_keys(&config.api_keys);
    if api_keys.is_empty() {
        tracing::warn!("SYNTAGMA_API_KEYS is not set — authentication is disabled (all endpoints open)");
    } else {
        tracing::info!("Authentication enabled with {} API key(s)", api_keys.len());
    }
    let app = create_app(
        graph,
        api_keys,
        &config.cors_origins,
        &config.redis_host,
        config.redis_port,
        config.redis_db,
        config.redis_ttl,
        config.redis_enabled,
        config.enable_debug_endpoints,
        config.telemetry_enabled,
        config.posthog_api_key.clone(),
        config.posthog_host.clone(),
        config.sentry_dsn.clone(),
    )
    .await;

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| anyhow::anyhow!("failed to bind: {e}"))?;

    tracing::info!("syntagma-api listening on {}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| anyhow::anyhow!("server error: {e}"))?;

    tracing::info!("syntagma-api stopped");
    Ok(())
}

/// Wait for a shutdown signal (ctrl-c).
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl+c");
    tracing::info!("shutdown signal received");
}
