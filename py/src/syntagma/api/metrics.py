#!/usr/bin/env python3
"""
Prometheus Metrics Configuration for Syntagma API
Provides request metrics, custom business metrics, and performance tracking
"""

from typing import Callable, Optional

from fastapi import FastAPI
from prometheus_client import Counter, Gauge, Histogram
from prometheus_fastapi_instrumentator import Instrumentator, metrics
from prometheus_fastapi_instrumentator.metrics import Info as MetricInfo

# ===== Custom Metrics =====

# Business metrics
smells_detected_total = Counter(
    "syntagma_smells_detected_total",
    "Total number of code smells detected",
    ["smell_id", "smell_name"],
)

searches_total = Counter(
    "syntagma_searches_total", "Total number of semantic searches", ["entity_type", "has_filter"]
)

refactoring_suggestions_total = Counter(
    "syntagma_refactoring_suggestions_total",
    "Total number of refactoring suggestions generated",
    ["refactoring_id"],
)

# Component health metrics
component_status = Gauge(
    "syntagma_component_status", "Component health status (1=healthy, 0=unhealthy)", ["component"]
)

# Performance metrics
analysis_duration = Histogram(
    "syntagma_analysis_duration_seconds",
    "Time spent analyzing code",
    buckets=[0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
)

search_duration = Histogram(
    "syntagma_search_duration_seconds",
    "Time spent on semantic search",
    buckets=[0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.0],
)

graph_query_duration = Histogram(
    "syntagma_graph_query_duration_seconds",
    "Time spent on graph queries",
    buckets=[0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5],
)

# Memory metrics
memory_usage_bytes = Gauge("syntagma_memory_usage_bytes", "Current memory usage in bytes")

# Active requests
active_requests = Gauge(
    "syntagma_active_requests",
    "Number of requests currently being processed",
    ["method", "endpoint"],
)


# ===== Custom Metric Functions for Instrumentator =====


def http_requests_total_custom() -> Callable[[MetricInfo], None]:
    """
    Custom request counter with endpoint and status labels
    """
    METRIC = Counter(
        "syntagma_http_requests_total",
        "Total number of HTTP requests",
        labelnames=("method", "endpoint", "status_code"),
    )

    def instrumentation(info: MetricInfo) -> None:
        endpoint = info.modified_handler or "unknown"
        METRIC.labels(method=info.method, endpoint=endpoint, status_code=info.modified_status).inc()

    return instrumentation


def http_request_errors_total() -> Callable[[MetricInfo], None]:
    """
    Counter for 4xx and 5xx errors
    """
    METRIC_4XX = Counter(
        "syntagma_http_errors_4xx_total",
        "Total number of 4xx client errors",
        labelnames=("method", "endpoint", "status_code"),
    )

    METRIC_5XX = Counter(
        "syntagma_http_errors_5xx_total",
        "Total number of 5xx server errors",
        labelnames=("method", "endpoint", "status_code"),
    )

    def instrumentation(info: MetricInfo) -> None:
        endpoint = info.modified_handler or "unknown"
        status = info.modified_status

        if 400 <= status < 500:
            METRIC_4XX.labels(method=info.method, endpoint=endpoint, status_code=status).inc()
        elif status >= 500:
            METRIC_5XX.labels(method=info.method, endpoint=endpoint, status_code=status).inc()

    return instrumentation


# ===== Instrumentator Setup =====


def setup_metrics(app: FastAPI) -> Instrumentator:
    """
    Configure Prometheus metrics for FastAPI app

    Args:
        app: FastAPI application instance

    Returns:
        Configured Instrumentator instance
    """
    instrumentator = Instrumentator(
        should_group_status_codes=False,
        should_ignore_untemplated=False,
        should_respect_env_var=True,
        should_instrument_requests_inprogress=True,
        excluded_handlers=["/metrics", "/health/live"],  # Don't track these
        env_var_name="ENABLE_METRICS",
        inprogress_name="http_requests_inprogress",
        inprogress_labels=True,
    )

    # Add default metrics
    instrumentator.add(metrics.default())
    instrumentator.add(metrics.latency(buckets=(0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10)))

    # Add custom metrics
    instrumentator.add(http_requests_total_custom())
    instrumentator.add(http_request_errors_total())

    # Instrument the app
    instrumentator.instrument(app)

    return instrumentator


# ===== Helper Functions =====


def update_component_health(component: str, is_healthy: bool):
    """
    Update component health status

    Args:
        component: Component name (smell_detector, graph, rag, etc.)
        is_healthy: True if component is healthy
    """
    component_status.labels(component=component).set(1 if is_healthy else 0)


def track_smell_detection(smell_id: str, smell_name: str):
    """
    Track a detected code smell

    Args:
        smell_id: Smell identifier (e.g., SMELL-01)
        smell_name: Smell name (e.g., Long Method)
    """
    smells_detected_total.labels(smell_id=smell_id, smell_name=smell_name).inc()


def track_search(entity_type: Optional[str] = None, has_filter: bool = False):
    """
    Track a semantic search query

    Args:
        entity_type: Type of entity being searched (or "all")
        has_filter: Whether filters were applied
    """
    searches_total.labels(
        entity_type=entity_type or "all", has_filter="yes" if has_filter else "no"
    ).inc()


def track_refactoring_suggestion(refactoring_id: str):
    """
    Track a refactoring suggestion

    Args:
        refactoring_id: Refactoring identifier (e.g., RF-001)
    """
    refactoring_suggestions_total.labels(refactoring_id=refactoring_id).inc()


def update_memory_usage():
    """
    Update current memory usage metric
    """
    try:
        import psutil

        process = psutil.Process()
        memory_info = process.memory_info()
        memory_usage_bytes.set(memory_info.rss)
    except ImportError:
        # psutil not available, skip
        pass
