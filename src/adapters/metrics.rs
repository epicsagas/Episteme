use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Instant;

use metrics_exporter_prometheus::PrometheusBuilder;

/// Global metrics handle, initialized exactly once.
static METRICS_HANDLE: OnceLock<Arc<MetricsHandle>> = OnceLock::new();

/// Handle to the Prometheus exporter. Kept alive for the lifetime of the
/// server so that the `/metrics` endpoint can call `handle.render()`.
pub struct MetricsHandle {
    handle: metrics_exporter_prometheus::PrometheusHandle,
}

impl MetricsHandle {
    /// Render the current Prometheus metrics snapshot as text.
    pub fn render(&self) -> String {
        self.handle.render()
    }
}

/// Install the Prometheus metrics recorder and return a handle that can render
/// metrics on demand.
///
/// Safe to call multiple times -- subsequent calls return a clone of the
/// handle that was created by the first call.
pub fn init_metrics() -> Arc<MetricsHandle> {
    METRICS_HANDLE
        .get_or_init(|| {
            let recorder = PrometheusBuilder::new().build_recorder();
            let handle = recorder.handle();

            // `OnceLock` guarantees this closure runs at most once, so the
            // `set_global_recorder` call here cannot race with itself.
            metrics::set_global_recorder(recorder)
                .expect("failed to install Prometheus metrics recorder");

            describe_metrics();

            Arc::new(MetricsHandle { handle })
        })
        .clone()
}

/// Register metric descriptions so they appear with `# HELP` / `# TYPE` lines
/// in the Prometheus output even before any values are recorded.
fn describe_metrics() {
    metrics::describe_counter!(
        "episteme_smells_detected_total",
        metrics::Unit::Count,
        "Total number of code smells detected by the analyze endpoint"
    );
    metrics::describe_counter!(
        "episteme_searches_total",
        metrics::Unit::Count,
        "Total number of search queries executed"
    );
    metrics::describe_counter!(
        "episteme_refactoring_suggestions_total",
        metrics::Unit::Count,
        "Total number of refactoring suggestions returned"
    );
    metrics::describe_histogram!(
        "episteme_analysis_duration_seconds",
        metrics::Unit::Seconds,
        "Duration of code analysis requests"
    );
    metrics::describe_histogram!(
        "episteme_search_duration_seconds",
        metrics::Unit::Seconds,
        "Duration of search requests"
    );
    metrics::describe_counter!(
        "episteme_rate_limit_rejected_total",
        metrics::Unit::Count,
        "Total number of requests rejected by the rate limiter"
    );
}

// ---------------------------------------------------------------------------
// Business-metric helpers
// ---------------------------------------------------------------------------

/// Record that one or more code smells were detected by the analyze endpoint.
pub fn track_smell_detection(smell_id: &str, smell_name: &str) {
    metrics::counter!(
        "episteme_smells_detected_total",
        "smell_id" => smell_id.to_owned(),
        "smell_name" => smell_name.to_owned(),
    )
    .increment(1);
}

/// Record a search query.
pub fn track_search(entity_type: &str, has_filter: bool) {
    metrics::counter!(
        "episteme_searches_total",
        "entity_type" => entity_type.to_owned(),
        "has_filter" => has_filter.to_string(),
    )
    .increment(1);
}

/// Record a refactoring suggestion.
pub fn track_refactoring_suggestion(refactoring_id: &str) {
    metrics::counter!(
        "episteme_refactoring_suggestions_total",
        "refactoring_id" => refactoring_id.to_owned(),
    )
    .increment(1);
}

/// Record that a request was rejected by the rate limiter.
pub fn track_rate_limit_rejected(path: &str) {
    metrics::counter!(
        "episteme_rate_limit_rejected_total",
        "path" => path.to_owned(),
    )
    .increment(1);
}

/// RAII guard that records the elapsed time on drop.
pub struct AnalysisTimer {
    start: Instant,
}

impl Default for AnalysisTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisTimer {
    /// Create a new timer. The elapsed duration is recorded when the timer is
    /// dropped.
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl Drop for AnalysisTimer {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed().as_secs_f64();
        metrics::histogram!("episteme_analysis_duration_seconds").record(elapsed);
    }
}

/// RAII guard that records search duration on drop.
pub struct SearchTimer {
    start: Instant,
}

impl Default for SearchTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchTimer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl Drop for SearchTimer {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed().as_secs_f64();
        metrics::histogram!("episteme_search_duration_seconds").record(elapsed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_metrics_does_not_panic() {
        // This test is safe to call once; repeated calls will panic because
        // `set_boxed_recorder` may only be called once. Guard with a
        // check.
        let _ = init_metrics();
    }

    #[test]
    fn track_smell_detection_increments_without_panic() {
        let _ = init_metrics();
        track_smell_detection("SMELL-001", "god_class");
    }

    #[test]
    fn track_search_increments_without_panic() {
        let _ = init_metrics();
        track_search("pattern", true);
    }

    #[test]
    fn track_refactoring_suggestion_increments_without_panic() {
        let _ = init_metrics();
        track_refactoring_suggestion("RF-001");
    }

    #[test]
    fn analysis_timer_records_on_drop() {
        let _ = init_metrics();
        {
            let _timer = AnalysisTimer::new();
            // timer dropped here, records elapsed
        }
    }

    #[test]
    fn search_timer_records_on_drop() {
        let _ = init_metrics();
        {
            let _timer = SearchTimer::new();
        }
    }
}
