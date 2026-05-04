use std::time::Instant;

pub fn log_business_event(event: &str, payload: serde_json::Value) {
    tracing::info!(
        event = event,
        payload = %payload,
        "business_event"
    );
}

pub fn log_request_started(request_id: &str, method: &str, path: &str) -> Instant {
    tracing::info!(
        request_id = request_id,
        method = method,
        path = path,
        "request_started"
    );
    Instant::now()
}

pub fn log_request_finished(
    request_id: &str,
    method: &str,
    path: &str,
    status: u16,
    started_at: Instant,
) {
    tracing::info!(
        request_id = request_id,
        method = method,
        path = path,
        status = status,
        duration_ms = started_at.elapsed().as_millis() as u64,
        "request_finished"
    );
}
