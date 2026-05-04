use axum::{
    extract::Request,
    http::HeaderValue,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

/// Global error handler for unhandled errors and panics.
///
/// Returns a structured JSON error response with a 500 status code.
/// This is used as the error handler for `CatchPanicLayer` so that
/// panics inside handlers do not terminate the server and instead
/// produce a clean error response.
pub fn global_error_handler(err: Box<dyn std::any::Any + Send + 'static>) -> Response {
    let panic_message = if let Some(s) = err.downcast_ref::<&str>() {
        (*s).to_owned()
    } else if let Some(s) = err.downcast_ref::<String>() {
        s.clone()
    } else {
        "panic".to_owned()
    };
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        axum::Json(serde_json::json!({
            "error": format!("internal server error: {panic_message}"),
            "status": 500,
        })),
    )
        .into_response()
}

/// Wrapper for API keys held in shared state.
#[derive(Clone, Default)]
pub struct ApiKeys(pub Vec<String>);

/// Build a CORS layer that allows all origins (suitable for development / API usage).
pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}

/// Inject a `X-Request-Id` header into the response if one is not already present.
pub async fn request_id_middleware(request: Request, next: Next) -> Response {
    let method = request.method().to_string();
    let path = request.uri().path().to_owned();
    let request_id = request
        .headers()
        .get("X-Request-Id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_owned())
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let started = crate::adapters::structured_logging::log_request_started(&request_id, &method, &path);

    let mut response = next.run(request).await;
    let status = response.status().as_u16();
    if let Ok(val) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert("X-Request-Id", val);
    }
    crate::adapters::structured_logging::log_request_finished(
        &request_id,
        &method,
        &path,
        status,
        started,
    );
    response
}

/// API-key authentication middleware.
///
/// - Skips auth for `/` and `/health*` endpoints.
/// - If no keys are configured (empty `ApiKeys`), all requests pass through (dev mode).
/// - Otherwise, requires a valid `X-API-Key` header matching one of the configured keys.
pub async fn auth_middleware(
    axum::extract::State(keys): axum::extract::State<Arc<ApiKeys>>,
    req: Request,
    next: Next,
) -> Response {
    let path = req.uri().path();
    let method = req.method();

    // Skip auth for health endpoints, root, and MCP GET convenience routes.
    if path == "/" || path.starts_with("/health") {
        return next.run(req).await;
    }

    // GET /tools and GET /resources are unauthenticated convenience endpoints.
    if method == axum::http::Method::GET && (path == "/tools" || path == "/resources") {
        return next.run(req).await;
    }

    // If no keys configured, skip auth (dev mode).
    if keys.0.is_empty() {
        return next.run(req).await;
    }

    // Extract X-API-Key header.
    let api_key = req.headers().get("X-API-Key").and_then(|v| v.to_str().ok());

    match api_key {
        Some(key) if crate::server::mcp_auth::validate_api_key(key, &keys.0) => {
            next.run(req).await
        }
        _ => {
            let mut response =
                (axum::http::StatusCode::UNAUTHORIZED, "Invalid or missing API key")
                    .into_response();
            response.headers_mut().insert(
                axum::http::header::WWW_AUTHENTICATE,
                "ApiKey".parse().unwrap(),
            );
            response
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Request as HttpRequest, StatusCode},
        middleware,
        routing::get,
    };
    use tower::ServiceExt;

    async fn ok_handler() -> &'static str {
        "ok"
    }

    fn make_app(keys: ApiKeys) -> Router {
        let state = Arc::new(keys);
        Router::new()
            .route("/", get(ok_handler))
            .route("/health", get(ok_handler))
            .route("/health/live", get(ok_handler))
            .route("/stats", get(ok_handler))
            .route("/mcp", get(ok_handler))
            .route("/tools", get(ok_handler))
            .route("/resources", get(ok_handler))
            .layer(middleware::from_fn_with_state(state, auth_middleware))
    }

    #[tokio::test]
    async fn empty_keys_skips_auth() {
        let app = make_app(ApiKeys(vec![]));
        let req = HttpRequest::builder()
            .uri("/stats")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn health_endpoint_bypasses_auth_without_key() {
        let app = make_app(ApiKeys(vec!["secret".into()]));
        let req = HttpRequest::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn health_live_endpoint_bypasses_auth_without_key() {
        let app = make_app(ApiKeys(vec!["secret".into()]));
        let req = HttpRequest::builder()
            .uri("/health/live")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn root_bypasses_auth_without_key() {
        let app = make_app(ApiKeys(vec!["secret".into()]));
        let req = HttpRequest::builder()
            .uri("/")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn valid_key_passes_through() {
        let app = make_app(ApiKeys(vec!["secret".into()]));
        let req = HttpRequest::builder()
            .uri("/stats")
            .header("X-API-Key", "secret")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn valid_key_among_multiple_passes_through() {
        let app = make_app(ApiKeys(vec![
            "alpha".into(),
            "beta".into(),
            "gamma".into(),
        ]));
        let req = HttpRequest::builder()
            .uri("/stats")
            .header("X-API-Key", "beta")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn invalid_key_returns_401() {
        let app = make_app(ApiKeys(vec!["secret".into()]));
        let req = HttpRequest::builder()
            .uri("/stats")
            .header("X-API-Key", "wrong")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        assert!(resp
            .headers()
            .get("WWW-Authenticate")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("ApiKey"));
    }

    #[tokio::test]
    async fn missing_key_returns_401() {
        let app = make_app(ApiKeys(vec!["secret".into()]));
        let req = HttpRequest::builder()
            .uri("/stats")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn mcp_endpoint_requires_key() {
        let app = make_app(ApiKeys(vec!["secret".into()]));
        let req = HttpRequest::builder()
            .uri("/mcp")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn tools_get_bypasses_auth_without_key() {
        let app = make_app(ApiKeys(vec!["secret".into()]));
        let req = HttpRequest::builder()
            .uri("/tools")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn resources_get_bypasses_auth_without_key() {
        let app = make_app(ApiKeys(vec!["secret".into()]));
        let req = HttpRequest::builder()
            .uri("/resources")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
