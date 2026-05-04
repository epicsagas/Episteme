# Syntagma Monitoring Setup

Prometheus + Grafana configuration for tracking Syntagma pattern usage and code quality metrics.

## Quick Start

### 1. Start Monitoring Stack

```bash
cd monitoring

# Using Docker Compose
docker-compose up -d
```

### 2. Instrument Your Code

```python
from prometheus_client import Counter, Histogram, Gauge, generate_latest
from fastapi import FastAPI
from fastapi.responses import Response

app = FastAPI()

# Define metrics
pattern_usage = Counter(
    'syntagma_pattern_applied_total',
    'Syntagma pattern application count',
    ['entity_id', 'entity_type', 'context', 'service', 'alcove_doc']
)

smell_detected = Counter(
    'syntagma_smell_detected_total',
    'Code smell detection count',
    ['smell_id', 'smell_name', 'severity', 'language']
)

refactoring_suggested = Counter(
    'syntagma_refactoring_suggested_total',
    'Refactoring suggestions generated',
    ['refactoring_id', 'refactoring_name']
)

refactoring_applied = Counter(
    'syntagma_refactoring_applied_total',
    'Refactoring suggestions applied by developers',
    ['refactoring_id', 'refactoring_name']
)

api_duration = Histogram(
    'syntagma_api_duration_seconds',
    'Syntagma API endpoint latency',
    ['endpoint', 'method', 'status']
)

api_requests = Counter(
    'syntagma_api_requests_total',
    'Total API requests',
    ['endpoint', 'method', 'status']
)

api_errors = Counter(
    'syntagma_api_errors_total',
    'API errors',
    ['endpoint', 'error_type']
)

# Expose /metrics endpoint
@app.get("/metrics")
async def metrics():
    return Response(content=generate_latest(), media_type="text/plain")
```

### 3. Track Pattern Usage

```python
# Example: Payment validation with Strategy pattern
from your_metrics import pattern_usage, smell_detected

def validate_payment(card_number: str):
    # Track pattern usage
    pattern_usage.labels(
        entity_id='DP-023',
        entity_type='pattern',
        context='payment_validation',
        service='payment-api',
        alcove_doc='DR-001'  # Link to Alcove decision record
    ).inc()
    
    # Your validation logic
    validator = CardValidator(strategy=StrategyA())
    result = validator.validate(card_number)
    
    # Track smell if detected
    if len(card_number) < 13:
        smell_detected.labels(
            smell_id='SMELL-42',
            smell_name='Data Validation',
            severity='high',
            language='python'
        ).inc()
    
    return result
```

### 4. Access Dashboards

- **Grafana**: http://localhost:3000 (admin/admin)
- **Prometheus**: http://localhost:9090

Import dashboard:
1. Go to Grafana → Dashboards → Import
2. Upload `grafana/syntagma-dashboard.json` (pattern/code-quality view) or `grafana/syntagma-api-dashboard.json` (API/latency/error view)

---

## Metrics Reference

### Pattern Usage Metrics

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `syntagma_pattern_applied_total` | Counter | entity_id, entity_type, context, service, alcove_doc | Pattern application count |
| `syntagma_pattern_usage_rate5m` | Gauge | entity_id | Pattern usage rate (5m window) |
| `syntagma_pattern_diversity_by_service` | Gauge | service | Unique patterns per service |

### Code Quality Metrics

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `syntagma_smell_detected_total` | Counter | smell_id, smell_name, severity, language | Code smell detections |
| `syntagma_smell_severity_count` | Gauge | severity | Smell count by severity |
| `syntagma_code_health_score` | Gauge | - | Overall code health (0-100) |

### Refactoring Metrics

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `syntagma_refactoring_suggested_total` | Counter | refactoring_id, refactoring_name | Suggestions generated |
| `syntagma_refactoring_applied_total` | Counter | refactoring_id, refactoring_name | Suggestions accepted |
| `syntagma_refactoring_acceptance_rate` | Gauge | - | Acceptance rate (%) |

### API Performance Metrics

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `syntagma_api_duration_seconds` | Histogram | endpoint, method, status | Request latency |
| `syntagma_api_requests_total` | Counter | endpoint, method, status | Total requests |
| `syntagma_api_errors_total` | Counter | endpoint, error_type | Error count |
| `syntagma_api_latency_p95` | Gauge | - | 95th percentile latency |
| `syntagma_api_latency_p99` | Gauge | - | 99th percentile latency |

---

## Alert Rules

### Banned Pattern Alert

Triggers when a banned pattern (e.g., Singleton per DR-091) is used:

```yaml
- alert: BannedPatternUsed
  expr: increase(syntagma_pattern_applied_total{entity_id="DP-006"}[5m]) > 0
  annotations:
    summary: "Singleton pattern used (banned per DR-091)"
    alcove_ref: "DR-091"
```

### Code Health Alert

Triggers when code health score drops below 70:

```yaml
- alert: LowCodeHealth
  expr: syntagma_code_health_score < 70
  annotations:
    summary: "Code health degraded (score: {{ $value }})"
```

### Pattern Usage Spike

Detects unusual pattern usage spikes (potential copy-paste):

```yaml
- alert: PatternUsageSpike
  expr: |
    rate(syntagma_pattern_applied_total[5m]) / 
    rate(syntagma_pattern_applied_total[1h] offset 1h) > 5
  annotations:
    summary: "Pattern {{ $labels.entity_id }} usage spiked 5x"
```

---

## Alcove Integration

### Link Metrics to Alcove Documents

Add `alcove_doc` label to track which team decision relates to each pattern:

```python
pattern_usage.labels(
    entity_id='DP-023',
    entity_type='pattern',
    context='payment_validation',
    service='payment-api',
    alcove_doc='DR-001'  # References DECISION.md DR-001
).inc()
```

### Generate Quarterly Reports

Query Prometheus and update Alcove docs:

```bash
#!/bin/bash
# scripts/update-alcove-usage-report.sh

PROMETHEUS_URL="http://localhost:9090"
ALCOVE_DOC=".alcove/PATTERN_USAGE.md"

# Fetch top 10 patterns
PATTERNS=$(curl -sG "$PROMETHEUS_URL/api/v1/query" \
  --data-urlencode 'query=topk(10, increase(syntagma_pattern_applied_total[90d]))' \
  | jq -r '.data.result[] | "\(.metric.entity_id):\(.value[1]):\(.metric.alcove_doc)"')

# Update Alcove document
cat > "$ALCOVE_DOC" <<EOF
# Pattern Usage Report ($(date +%Y-Q%q))

Generated: $(date)

## Top Patterns

EOF

echo "$PATTERNS" | while IFS=: read -r entity count alcove_doc; do
  cat >> "$ALCOVE_DOC" <<EOF
### $entity ($count uses)
- Team reference: $alcove_doc
- Syntagma: https://syntagma.dev/entities/$entity

EOF
done

# Commit to Alcove
alcove rebuild-index .
```

---

## Docker Compose Setup

```yaml
# monitoring/docker-compose.yml
version: '3.8'

services:
  prometheus:
    image: prom/prometheus:latest
    container_name: syntagma-prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus/prometheus.yml:/etc/prometheus/prometheus.yml
      - ./prometheus/syntagma-rules.yml:/etc/prometheus/syntagma-rules.yml
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/usr/share/prometheus/console_libraries'
      - '--web.console.templates=/usr/share/prometheus/consoles'

  grafana:
    image: grafana/grafana:latest
    container_name: syntagma-grafana
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
      - GF_USERS_ALLOW_SIGN_UP=false
    volumes:
      - grafana_data:/var/lib/grafana
      - ./grafana/provisioning:/etc/grafana/provisioning
    depends_on:
      - prometheus

  alertmanager:
    image: prom/alertmanager:latest
    container_name: syntagma-alertmanager
    ports:
      - "9093:9093"
    volumes:
      - ./alertmanager/config.yml:/etc/alertmanager/config.yml
      - alertmanager_data:/alertmanager

volumes:
  prometheus_data:
  grafana_data:
  alertmanager_data:
```

---

## Troubleshooting

### Metrics not showing up

1. Check Prometheus targets: http://localhost:9090/targets
2. Verify `/metrics` endpoint is accessible:
   ```bash
   curl http://localhost:8000/metrics
   ```
3. Check label cardinality (avoid high-cardinality labels like user_id)

### Dashboard empty

1. Verify time range (top-right corner in Grafana)
2. Check Prometheus data source: Configuration → Data Sources
3. Validate PromQL queries in Prometheus UI first

### Alerts not firing

1. Check alert rules status: http://localhost:9090/alerts
2. Verify Alertmanager config: http://localhost:9093
3. Test alert expression in Prometheus query browser

---

## Best Practices

### 1. Label Hygiene

```python
# ✅ Good: bounded label cardinality
pattern_usage.labels(
    entity_id='DP-023',      # ~100 unique values
    context='payment',        # ~20 unique values
    service='payment-api'     # ~10 unique values
).inc()

# ❌ Bad: unbounded label cardinality
pattern_usage.labels(
    entity_id='DP-023',
    user_id=str(user.id),     # Millions of unique values → ⚠️ Cardinality explosion
    timestamp=str(now())      # Infinite unique values
).inc()
```

### 2. Metric Naming Convention

Follow Prometheus naming best practices:

- `syntagma_<noun>_<unit>_total` for counters
- `syntagma_<noun>_<unit>` for gauges/histograms
- Use base units (seconds, bytes, not milliseconds/KB)

### 3. Recording Rules

Pre-compute expensive queries:

```yaml
# Good: pre-compute top patterns daily
- record: syntagma:pattern_usage:top10_daily
  expr: topk(10, increase(syntagma_pattern_applied_total[24h]))
  
# Then query the recording rule instead of raw metric
```

---

## Further Reading

- [Prometheus Best Practices](https://prometheus.io/docs/practices/naming/)
- [Grafana Dashboard Best Practices](https://grafana.com/docs/grafana/latest/dashboards/build-dashboards/best-practices/)
- [Alcove Integration Guide](../docs/alcove-integration.md)
