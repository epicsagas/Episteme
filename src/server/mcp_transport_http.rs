use axum::{
    Json, Router,
    extract::Request,
    extract::State,
    http::StatusCode,
    middleware::{self, Next},
    response::IntoResponse,
    response::Response,
    routing::{get, post},
};
use serde::Deserialize;
use std::sync::Arc;

use crate::adapters::constants::MAX_REQUEST_BYTES;
use crate::server::mcp_dispatcher::dispatch;
use crate::server::mcp_handler::SyntagmaMCP;

pub type SharedMCP = Arc<SyntagmaMCP>;
#[derive(Clone)]
pub struct McpAuthKeys(pub Vec<String>);

/// Build an axum Router that serves MCP endpoints over HTTP.
///
/// Routes:
/// - `POST /mcp`  — full JSON-RPC protocol
/// - `GET  /health` — simple health check
/// - `GET  /tools` — tool schema list
/// - `GET  /resources` — resource schema list
/// - `POST /tool` — convenience tool-call (no JSON-RPC envelope required)
pub fn mcp_http_router(mcp: SyntagmaMCP, allowed_api_keys: Vec<String>) -> Router {
    let state: SharedMCP = Arc::new(mcp);
    Router::new()
        .route("/mcp", post(handle_mcp_post))
        .route("/health", get(handle_health))
        .route("/tools", get(handle_tools_list))
        .route("/resources", get(handle_resources_list))
        .route("/tool", post(handle_tool_call))
        .with_state(state)
        .layer(middleware::from_fn_with_state(
            Arc::new(McpAuthKeys(allowed_api_keys)),
            mcp_auth_middleware,
        ))
        // Enforce a body-size limit on all routes.
        .layer(axum::extract::DefaultBodyLimit::max(MAX_REQUEST_BYTES))
}

async fn mcp_auth_middleware(
    State(keys): State<Arc<McpAuthKeys>>,
    request: Request,
    next: Next,
) -> Response {
    if request.uri().path() == "/health" {
        return next.run(request).await;
    }
    if keys.0.is_empty() {
        return next.run(request).await;
    }
    let bearer = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));
    match bearer {
        Some(token) if crate::server::mcp_auth::validate_api_key(token, &keys.0) => {
            next.run(request).await
        }
        _ => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "unauthorized"})),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// GET convenience endpoints
// ---------------------------------------------------------------------------

/// `GET /health` -- simple liveness check.
pub async fn handle_health() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

/// `GET /tools` -- return the MCP tool schemas as a JSON array.
pub async fn handle_tools_list() -> impl IntoResponse {
    Json(crate::server::mcp_schemas::tool_schemas())
}

/// `GET /resources` -- return the MCP resource schemas as a JSON array.
pub async fn handle_resources_list() -> impl IntoResponse {
    Json(crate::server::mcp_schemas::resource_schemas())
}

// ---------------------------------------------------------------------------
// POST endpoints
// ---------------------------------------------------------------------------

/// Handle a single JSON-RPC request over HTTP (`POST /mcp`).
pub async fn handle_mcp_post(
    State(mcp): State<SharedMCP>,
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    // Validate that the request is a JSON object
    if !request.is_object() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "jsonrpc": "2.0",
                "error": { "code": -32600, "message": "Invalid Request: expected JSON object" },
                "id": null,
            })),
        );
    }

    match dispatch(&mcp, request) {
        Some(response) => (StatusCode::OK, Json(response)),
        None => (StatusCode::NO_CONTENT, Json(serde_json::Value::Null)),
    }
}

/// Convenience `POST /tool` -- accepts `{name, arguments}` and returns just the result.
///
/// The caller does not need to construct a full JSON-RPC envelope.
#[derive(Deserialize)]
pub struct ToolCallRequest {
    pub name: String,
    #[serde(default)]
    pub arguments: serde_json::Value,
}

pub async fn handle_tool_call(
    State(mcp): State<SharedMCP>,
    Json(body): Json<ToolCallRequest>,
) -> impl IntoResponse {
    let result = mcp.handle_tool_call(&body.name, &body.arguments);

    if let Some(err) = result.get("error") {
        let msg = err.as_str().unwrap_or("tool call error");
        let status = if msg.contains("Unknown tool") {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::BAD_REQUEST
        };
        return (status, Json(result));
    }

    (StatusCode::OK, Json(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    fn test_router(keys: Vec<String>) -> Router {
        let graph =
            crate::domain::graph::KnowledgeGraph::from_entities(std::collections::HashMap::new());
        let mcp = crate::server::mcp_handler::SyntagmaMCP::new(graph);
        mcp_http_router(mcp, keys)
    }

    #[tokio::test]
    async fn health_is_public() {
        let app = test_router(vec!["k1".to_string()]);
        let req = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn tools_requires_bearer_when_keys_present() {
        let app = test_router(vec!["k1".to_string()]);
        let req = Request::builder()
            .uri("/tools")
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn tools_allows_valid_bearer() {
        let app = test_router(vec!["k1".to_string()]);
        let req = Request::builder()
            .uri("/tools")
            .header(axum::http::header::AUTHORIZATION, "Bearer k1")
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
}
