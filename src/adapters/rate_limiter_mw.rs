use std::sync::Arc;

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::adapters::rate_limiter::{RateLimiter, rate_limit_for_path};

/// Per-route rate-limiting middleware for axum.
///
/// Extracts a client key from `X-Forwarded-For` (first entry) or falls back to
/// `"unknown"`, then checks a token bucket whose capacity is determined by the
/// request path. Returns `429 Too Many Requests` when the limit is exceeded.
pub async fn rate_limit_middleware(
    axum::extract::State(limiter): axum::extract::State<Arc<RateLimiter>>,
    req: Request,
    next: Next,
) -> Response {
    let path = req.uri().path().to_owned();

    let limit = rate_limit_for_path(&path);

    // The middleware receives a single generic `RateLimiter`. To enforce
    // per-route limits we dynamically reconfigure the limiter per-request by
    // using the combination of IP and limit as the bucket key. This means each
    // (client, route-tier) pair gets its own independent bucket.
    let client_ip = extract_client_ip(&req);
    let bucket_key = format!("{client_ip}:{limit}");

    if !limiter.allow(&bucket_key) {
        tracing::warn!(path = %path, client = %client_ip, "rate limit exceeded");
        crate::adapters::metrics::track_rate_limit_rejected(&path);
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded. Please try again later.",
        )
            .into_response();
    }

    next.run(req).await
}

/// Extract the client IP from `X-Forwarded-For` or fall back to `"unknown"`.
///
/// When running behind a reverse proxy the `X-Forwarded-For` header contains
/// the original client IP as its first entry. If the header is absent we use
/// a static fallback key so all un-proxied clients share a single bucket.
fn extract_client_ip(req: &Request) -> String {
    req.headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|s| s.trim().to_owned())
        .unwrap_or_else(|| "unknown".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, body::Body, http::Request as HttpRequest, middleware, routing::get};
    use tower::ServiceExt;

    async fn ok_handler() -> &'static str {
        "ok"
    }

    fn make_app(limiter: Arc<RateLimiter>) -> Router {
        Router::new()
            .route("/analyze", get(ok_handler))
            .route("/health", get(ok_handler))
            .layer(middleware::from_fn_with_state(
                limiter,
                rate_limit_middleware,
            ))
    }

    #[tokio::test]
    async fn allows_requests_within_limit() {
        let limiter = Arc::new(RateLimiter::new(20));
        let app = make_app(limiter);
        let req = HttpRequest::builder()
            .uri("/analyze")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn rejects_requests_over_limit() {
        // Create a limiter with capacity 1.
        let exhausted = Arc::new(RateLimiter::new(1));
        let app2 = make_app(Arc::clone(&exhausted));

        // First request passes.
        let req_a = HttpRequest::builder()
            .uri("/analyze")
            .body(Body::empty())
            .unwrap();
        let resp_a = app2.oneshot(req_a).await.unwrap();
        assert_eq!(resp_a.status(), StatusCode::OK);

        // Build a new app sharing the same exhausted limiter.
        let app3 = make_app(exhausted);
        let req_b = HttpRequest::builder()
            .uri("/analyze")
            .body(Body::empty())
            .unwrap();
        let resp_b = app3.oneshot(req_b).await.unwrap();
        assert_eq!(resp_b.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn forwarded_for_extracts_first_ip() {
        let req = HttpRequest::builder()
            .header("x-forwarded-for", "1.2.3.4, 5.6.7.8")
            .body(Body::empty())
            .unwrap();
        let ip = extract_client_ip(&req);
        assert_eq!(ip, "1.2.3.4");
    }

    #[tokio::test]
    async fn missing_forwarded_for_falls_back_to_unknown() {
        let req = HttpRequest::builder().body(Body::empty()).unwrap();
        let ip = extract_client_ip(&req);
        assert_eq!(ip, "unknown");
    }
}
