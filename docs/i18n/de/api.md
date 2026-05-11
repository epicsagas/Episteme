# Episteme REST-API-Dokumentation

**Version:** 0.1.0
**Basis-URL:** `http://localhost:8000`

---

## Schnellstart

```bash
# Server starten
epis api

# Oder mit benutzerdefiniertem Host/Port
epis api --host 0.0.0.0 --port 8000

# Docker
docker-compose up -d
```

---

## Authentifizierung

Alle Endpunkte außer `/`, `/health`, `/live`, `/ready` erfordern eine API-Schlüssel-Authentifizierung.

### API-Schlüssel-Authentifizierung

**Header:** `X-API-Key: <your-api-key>`

**Modi:**

1. **Produktionsmodus** - Umgebungsvariable `EPISTEME_API_KEYS` setzen
   - Kommagetrennte Liste gültiger API-Schlüssel
   - Alle geschützten Endpunkte erfordern einen gültigen Schlüssel
   - Gibt 401 Unauthorized zurück, falls fehlend/ungültig

2. **Entwicklungsmodus** - `EPISTEME_API_KEYS` leer lassen oder nicht setzen
   - Keine Authentifizierung erforderlich

### API-Schlüssel generieren

```bash
openssl rand -base64 32
```

### Beispielanfragen

```bash
# Mit Authentifizierung (Produktion)
curl -X POST http://localhost:8000/analyze \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{"code": "def long_method(): pass", "min_confidence": 0.5}'

# Ohne Authentifizierung (Entwicklungsmodus)
curl -X POST http://localhost:8000/analyze \
  -H "Content-Type: application/json" \
  -d '{"code": "def long_method(): pass"}'
```

---

## Rate-Limiting

Alle Endpunkte sind pro IP-Adresse mit TTL-basierter Bucket-Eviction rate-limitiert.

| Endpunkt | Rate-Limit | Grund |
|-----------|------------|-------|
| `/analyze` | 20/Minute | CPU-intensiv |
| `/refactor` | 20/Minute | CPU-intensiv |
| `/search` | 50/Minute | Embedding-Berechnung |
| `/stats`, `/graph/*` | 100/Minute | Standard |
| `/`, `/health` | Unbegrenzt | Öffentlich |

Bei Überschreitung wird 429 mit `Retry-After`-Header zurückgegeben.

---

## Endpunkte

### Zustand & Informationen

#### `GET /`

Dienstinformationen.

**Antwort:**
```json
{
  "name": "episteme",
  "version": "0.1.0",
  "description": "Software engineering knowledge graph",
  "endpoints": ["analyze", "search", "graph", "refactor", "stats"]
}
```

#### `GET /health`

Zustandsprüfung mit Komponentenstatus.

**Antwort:**
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

Verfügbarkeitsprüfung: `{"status": "alive"}`

#### `GET /ready`

Bereitschaftsprüfung: `{"status": "ready"}` (503 wenn nicht bereit)

#### `GET /stats`

Graph-Statistiken.

**Antwort:**
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

### Code-Analyse

#### Unterstützte Code-Smells (16 Detektoren)

| ID | Name | Sprachen |
|---|---|---|
| SMELL-01 | Long Method | Alle |
| SMELL-02 | Long Parameter List | Alle |
| SMELL-03 | Primitive Obsession | Python |
| SMELL-04 | Large Class | Alle |
| SMELL-05 | Data Clumps | Alle (Stub) |
| SMELL-06 | Switch Statements | Alle |
| SMELL-07 | Data Class | Alle |
| SMELL-09 | Shotgun Surgery | Alle (Stub) |
| SMELL-10 | Divergent Change | Alle |
| SMELL-11 | Lazy Class | Alle |
| SMELL-12 | Speculative Generality | Alle |
| SMELL-13 | Duplicate Code | Alle (teilweise) |
| SMELL-14 | Middle Man | Alle |
| SMELL-18 | Feature Envy | Alle |
| SMELL-20 | Message Chains | Alle |
| SMELL-21 | God Object | Alle |

#### `POST /analyze`

Code-Smells erkennen.

**Anfrage:**
```json
{
  "code": "def long_method():\n    ...",
  "language": "python",
  "min_confidence": 0.5
}
```

**Antwort:**
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

Rangfolgebasierte Refactoring-Vorschläge für erkannte Smells abrufen.

**Anfrage:**
```json
{
  "code": "def long_method():\n    ...",
  "top_k": 3,
  "min_confidence": 0.5
}
```

**Antwort:**
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

### Suche

#### `GET /search`

Suche über Abfrageparameter: `/search?q=strategy+pattern&top_k=5`

#### `POST /search`

Semantische Suche über der Wissensbasis.

**Anfrage:**
```json
{
  "query": "How to fix Long Method?",
  "top_k": 5,
  "entity_type": "refactoring"
}
```

**Antwort:**
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

### Wissensgraph

#### `GET /graph/{id}`

Entitätsdetails nach ID abrufen.

**Beispiel:** `GET /graph/DP-005`

#### `GET /graph/{id}/neighbors`

Nachbarn einer Entität abrufen: `/graph/SMELL-01/neighbors?relation_type=solved_by`

#### `POST /graph/neighbors`

Nachbarn abrufen (POST).

**Anfrage:**
```json
{
  "entity_id": "SMELL-01",
  "relation_type": "solved_by"
}
```

#### `GET /graph/path`

Kürzester Pfad: `/graph/path?from_id=SMELL-01&to_id=LAW-042-S&max_depth=5`

#### `POST /graph/subgraph`

Subgraph extrahieren.

**Anfrage:**
```json
{
  "entity_id": "DP-005",
  "depth": 2
}
```

#### `GET /graph/contradictions`

Entitäten mit widersprüchlichen Relationen finden.

#### `POST /graph/infer-transitive`

Transitive Durchsetzungsbeziehungen ableiten.

---

### Monitoring

#### `GET /metrics`

Prometheus-formatierte Metriken, einschließlich:
- `http_requests_total` — nach Methode, Endpunkt, Status
- `episteme_smells_detected_total` — nach smell_id
- `episteme_searches_total` — nach entity_type
- `episteme_analysis_duration_seconds` — Histogramm

---

## Leistung

| Endpunkt | Durchschnittliche Latenz | Hinweise |
|-----------|-------------------------|----------|
| `/analyze` | ~5ms | Regex + AST-Parsing (OnceLock-gecacht) |
| `/refactor` | ~10ms | Enthält Graph-Traversierung |
| `/search` | ~20ms | FTS5 + Kosinusähnlichkeit |
| `/graph/neighbors` | ~1ms | In-Memory-Graph |
| `/graph/path` | ~5ms | BFS bis Tiefe 5 |

---

## Fehlerbehandlung

| Status | Bedeutung |
|--------|-----------|
| 200 | Erfolg |
| 400 | Ungültige Anfrage |
| 401 | Fehhlender/ungültiger API-Schlüssel |
| 404 | Entität nicht gefunden |
| 429 | Rate-Limit überschritten |
| 500 | Interner Fehler |

---

## Umgebungsvariablen

```bash
# Server
EPISTEME_API_HOST=0.0.0.0
EPISTEME_API_PORT=8000
EPISTEME_API_KEYS=key1,key2

# Daten
EPISTEME_DATA_DIR=~/.episteme/data
EPISTEME_DB_PATH=~/.episteme/db/episteme.db

# Protokollierung
RUST_LOG=info
```

---

## Lizenz

APACHE-2.0-Lizenz
