# Episteme REST API Documentation

**Version:** 0.1.0
**Base URL:** `http://localhost:8000`

---

## Quick Start

```bash
# Start server
epis api

# Or with custom host/port
epis api --host 0.0.0.0 --port 8000

# Docker
docker-compose up -d
```

---

## Authentication

All endpoints except `/`, `/health`, `/live`, `/ready` require API key authentication.

### API Key Authentication

**Header:** `X-API-Key: <your-api-key>`

**Modes:**

1. **Production Mode** - Set `EPISTEME_API_KEYS` environment variable
   - Comma-separated list of valid API keys
   - All protected endpoints require valid key
   - Returns 401 Unauthorized if missing/invalid

2. **Development Mode** - Leave `EPISTEME_API_KEYS` empty or unset
   - No authentication required

### Generating API Keys

```bash
openssl rand -base64 32
```

### Example Requests

```bash
# With authentication (production)
curl -X POST http://localhost:8000/analyze \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{"code": "def long_method(): pass", "min_confidence": 0.5}'

# Without authentication (dev mode)
curl -X POST http://localhost:8000/analyze \
  -H "Content-Type: application/json" \
  -d '{"code": "def long_method(): pass"}'
```

---

## Rate Limiting

All endpoints are rate-limited per IP address with TTL-based bucket eviction.

| Endpoint | Rate Limit | Reason |
|----------|------------|--------|
| `/analyze` | 20/minute | CPU-intensive |
| `/refactor` | 20/minute | CPU-intensive |
| `/search` | 50/minute | Embedding computation |
| `/stats`, `/graph/*` | 100/minute | Standard |
| `/`, `/health` | Unlimited | Public |

When exceeded, returns 429 with `Retry-After` header.

---

## Endpoints

### Health & Info

#### `GET /`

Service information.

**Response:**
```json
{
  "name": "episteme",
  "version": "0.1.0",
  "description": "Software engineering knowledge graph",
  "endpoints": ["analyze", "search", "graph", "refactor", "stats"]
}
```

#### `GET /health`

Health check with component status.

**Response:**
```json
{
  "status": "healthy",
  "components": {
    "knowledge_graph": "ok",
    "rag_database": "ok",
    "embedding_provider": "local"
  }
}
```

#### `GET /live`

Liveness probe: `{"status": "alive"}`

#### `GET /ready`

Readiness probe: `{"status": "ready"}` (503 if not ready)

#### `GET /stats`

Graph statistics.

**Response:**
```json
{
  "total_entities": 161,
  "total_edges": 201,
  "by_type": {
    "refactoring": 66,
    "law": 56,
    "pattern": 22,
    "smell": 17
  }
}
```

---

### Code Analysis

#### Supported Code Smells (16 detectors)

| ID | Name | Languages |
|---|---|---|
| SMELL-01 | Long Method | All |
| SMELL-02 | Long Parameter List | All |
| SMELL-03 | Primitive Obsession | Python |
| SMELL-04 | Large Class | All |
| SMELL-05 | Data Clumps | All (stub) |
| SMELL-06 | Switch Statements | All |
| SMELL-07 | Data Class | All |
| SMELL-09 | Shotgun Surgery | All (stub) |
| SMELL-10 | Divergent Change | All |
| SMELL-11 | Lazy Class | All |
| SMELL-12 | Speculative Generality | All |
| SMELL-13 | Duplicate Code | All (partial) |
| SMELL-14 | Middle Man | All |
| SMELL-18 | Feature Envy | All |
| SMELL-20 | Message Chains | All |
| SMELL-21 | God Object | All |

#### `POST /analyze`

Detect code smells.

**Request:**
```json
{
  "code": "def long_method():\n    ...",
  "language": "python",
  "min_confidence": 0.5
}
```

**Response:**
```json
{
  "count": 2,
  "smells": [
    {
      "smell_id": "SMELL-01",
      "smell_name": "Long Method",
      "confidence": 0.90,
      "location": "temp.py:1",
      "function_name": "long_method",
      "metrics": {
        "loc": 94,
        "cyclomatic_complexity": 27,
        "nesting_depth": 5,
        "parameter_count": 9
      },
      "reasons": ["LOC=94 exceeds 30", "CC=27 exceeds 10"]
    }
  ]
}
```

#### `POST /refactor`

Get ranked refactoring suggestions for detected smells.

**Request:**
```json
{
  "code": "def long_method():\n    ...",
  "top_k": 3,
  "min_confidence": 0.5
}
```

**Response:**
```json
{
  "count": 1,
  "analyses": [
    {
      "smell": { "smell_id": "SMELL-01", "smell_name": "Long Method" },
      "suggestions": [
        {
          "refactoring_id": "RF-001",
          "title": "Extract Method",
          "priority_score": 0.79,
          "effort": "medium",
          "principles_enforced": ["LAW-040", "LAW-042-S"]
        }
      ]
    }
  ]
}
```

---

### Search

#### `GET /search`

Search via query parameter: `/search?q=strategy+pattern&top_k=5`

#### `POST /search`

Semantic search over knowledge base.

**Request:**
```json
{
  "query": "How to fix Long Method?",
  "top_k": 5,
  "entity_type": "refactoring"
}
```

**Response:**
```json
{
  "count": 3,
  "results": [
    {
      "entity_id": "RF-001",
      "title": "Extract Method",
      "category": "refactoring",
      "similarity": 0.85,
      "content": "Extract Method is a refactoring technique..."
    }
  ]
}
```

---

### Knowledge Graph

#### `GET /graph/{id}`

Get entity details by ID.

**Example:** `GET /graph/DP-005`

#### `GET /graph/{id}/neighbors`

Get neighbors of an entity: `/graph/SMELL-01/neighbors?relation_type=solved_by`

#### `POST /graph/neighbors`

Get neighbors (POST).

**Request:**
```json
{
  "entity_id": "SMELL-01",
  "relation_type": "solved_by"
}
```

#### `GET /graph/path`

Shortest path: `/graph/path?from_id=SMELL-01&to_id=LAW-042-S&max_depth=5`

#### `POST /graph/subgraph`

Extract subgraph.

**Request:**
```json
{
  "entity_id": "DP-005",
  "depth": 2
}
```

#### `GET /graph/contradictions`

Find entities with conflicting relations.

#### `POST /graph/infer-transitive`

Infer transitive enforcement relationships.

---

### Monitoring

#### `GET /metrics`

Prometheus-format metrics including:
- `http_requests_total` — by method, endpoint, status
- `episteme_smells_detected_total` — by smell_id
- `episteme_searches_total` — by entity_type
- `episteme_analysis_duration_seconds` — histogram

---

## Performance

| Endpoint | Avg Latency | Notes |
|----------|-------------|-------|
| `/analyze` | ~5ms | Regex + AST parsing (OnceLock cached) |
| `/refactor` | ~10ms | Includes graph traversal |
| `/search` | ~20ms | FTS5 + cosine similarity |
| `/graph/neighbors` | ~1ms | In-memory graph |
| `/graph/path` | ~5ms | BFS up to depth 5 |

---

## Error Handling

| Status | Meaning |
|--------|---------|
| 200 | Success |
| 400 | Bad request |
| 401 | Missing/invalid API key |
| 404 | Entity not found |
| 429 | Rate limit exceeded |
| 500 | Internal error |

---

## Environment Variables

```bash
# Server
EPISTEME_API_HOST=0.0.0.0
EPISTEME_API_PORT=8000
EPISTEME_API_KEYS=key1,key2

# Data
EPISTEME_DATA_DIR=~/.episteme/data
EPISTEME_DB_PATH=~/.episteme/db/episteme.db

# Logging
RUST_LOG=info
```

---

## License

APACHE-2.0 License
