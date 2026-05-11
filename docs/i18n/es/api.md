# Documentacion de la API REST Episteme

**Version:** 0.1.0
**URL base:** `http://localhost:8000`

---

## Inicio rapido

```bash
# Iniciar servidor
epis api

# O con host/puerto personalizado
epis api --host 0.0.0.0 --port 8000

# Docker
docker-compose up -d
```

---

## Autenticacion

Todos los endpoints excepto `/`, `/health`, `/live`, `/ready` requieren autenticacion por clave API.

### Autenticacion por clave API

**Encabezado:** `X-API-Key: <su-clave-api>`

**Modos:**

1. **Modo Produccion** - Establecer la variable de entorno `EPISTEME_API_KEYS`
   - Lista separada por comas de claves API validas
   - Todos los endpoints protegidos requieren una clave valida
   - Retorna 401 Unauthorized si falta/es invalida

2. **Modo Desarrollo** - Dejar `EPISTEME_API_KEYS` vacia o sin definir
   - Sin autenticacion requerida

### Generar claves API

```bash
openssl rand -base64 32
```

### Ejemplos de solicitudes

```bash
# Con autenticacion (produccion)
curl -X POST http://localhost:8000/analyze \
  -H "Content-Type: application/json" \
  -H "X-API-Key: su-clave-api" \
  -d '{"code": "def long_method(): pass", "min_confidence": 0.5}'

# Sin autenticacion (modo dev)
curl -X POST http://localhost:8000/analyze \
  -H "Content-Type: application/json" \
  -d '{"code": "def long_method(): pass"}'
```

---

## Limitacion de tasa

Todos los endpoints estan limitados por tasa por direccion IP con eviccion de cubos basada en TTL.

| Endpoint | Limite de tasa | Razon |
|----------|----------------|-------|
| `/analyze` | 20/minuto | Intensivo en CPU |
| `/refactor` | 20/minuto | Intensivo en CPU |
| `/search` | 50/minuto | Computo de embeddings |
| `/stats`, `/graph/*` | 100/minuto | Estandar |
| `/`, `/health` | Ilimitado | Publico |

Cuando se excede, retorna 429 con el encabezado `Retry-After`.

---

## Endpoints

### Salud e informacion

#### `GET /`

Informacion del servicio.

**Respuesta:**
```json
{
  "name": "episteme",
  "version": "0.1.0",
  "description": "Software engineering knowledge graph",
  "endpoints": ["analyze", "search", "graph", "refactor", "stats"]
}
```

#### `GET /health`

Verificacion de salud con estado de componentes.

**Respuesta:**
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

Sonda de vivacidad: `{"status": "alive"}`

#### `GET /ready`

Sonda de disponibilidad: `{"status": "ready"}` (503 si no esta listo)

#### `GET /stats`

Estadisticas del grafo.

**Respuesta:**
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

### Analisis de codigo

#### Code smells soportados (16 detectores)

| ID | Nombre | Lenguajes |
|---|---|---|
| SMELL-01 | Long Method | Todos |
| SMELL-02 | Long Parameter List | Todos |
| SMELL-03 | Primitive Obsession | Python |
| SMELL-04 | Large Class | Todos |
| SMELL-05 | Data Clumps | Todos (stub) |
| SMELL-06 | Switch Statements | Todos |
| SMELL-07 | Data Class | Todos |
| SMELL-09 | Shotgun Surgery | Todos (stub) |
| SMELL-10 | Divergent Change | Todos |
| SMELL-11 | Lazy Class | Todos |
| SMELL-12 | Speculative Generality | Todos |
| SMELL-13 | Duplicate Code | Todos (parcial) |
| SMELL-14 | Middle Man | Todos |
| SMELL-18 | Feature Envy | Todos |
| SMELL-20 | Message Chains | Todos |
| SMELL-21 | God Object | Todos |

#### `POST /analyze`

Detectar code smells.

**Solicitud:**
```json
{
  "code": "def long_method():\n    ...",
  "language": "python",
  "min_confidence": 0.5
}
```

**Respuesta:**
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

Obtener sugerencias de refactoring clasificadas para los smells detectados.

**Solicitud:**
```json
{
  "code": "def long_method():\n    ...",
  "top_k": 3,
  "min_confidence": 0.5
}
```

**Respuesta:**
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

### Busqueda

#### `GET /search`

Busqueda via parametro de consulta: `/search?q=strategy+pattern&top_k=5`

#### `POST /search`

Busqueda semantica en la base de conocimiento.

**Solicitud:**
```json
{
  "query": "How to fix Long Method?",
  "top_k": 5,
  "entity_type": "refactoring"
}
```

**Respuesta:**
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

### Grafo de conocimiento

#### `GET /graph/{id}`

Obtener detalles de una entidad por su ID.

**Ejemplo:** `GET /graph/DP-005`

#### `GET /graph/{id}/neighbors`

Obtener vecinos de una entidad: `/graph/SMELL-01/neighbors?relation_type=solved_by`

#### `POST /graph/neighbors`

Obtener vecinos (POST).

**Solicitud:**
```json
{
  "entity_id": "SMELL-01",
  "relation_type": "solved_by"
}
```

#### `GET /graph/path`

Camino mas corto: `/graph/path?from_id=SMELL-01&to_id=LAW-042-S&max_depth=5`

#### `POST /graph/subgraph`

Extraer subgrafo.

**Solicitud:**
```json
{
  "entity_id": "DP-005",
  "depth": 2
}
```

#### `GET /graph/contradictions`

Encontrar entidades con relaciones conflictivas.

#### `POST /graph/infer-transitive`

Inferir relaciones de aplicacion transitiva.

---

### Monitoreo

#### `GET /metrics`

Metricas en formato Prometheus incluyendo:
- `http_requests_total` — por metodo, endpoint, estado
- `episteme_smells_detected_total` — por smell_id
- `episteme_searches_total` — por entity_type
- `episteme_analysis_duration_seconds` — histograma

---

## Rendimiento

| Endpoint | Latencia promedio | Notas |
|----------|-------------------|-------|
| `/analyze` | ~5ms | Parsing regex + AST (cache OnceLock) |
| `/refactor` | ~10ms | Incluye recorrido de grafo |
| `/search` | ~20ms | FTS5 + similitud coseno |
| `/graph/neighbors` | ~1ms | Grafo en memoria |
| `/graph/path` | ~5ms | BFS hasta profundidad 5 |

---

## Manejo de errores

| Estado | Significado |
|--------|-------------|
| 200 | Exito |
| 400 | Solicitud incorrecta |
| 401 | Clave API faltante/invalida |
| 404 | Entidad no encontrada |
| 429 | Limite de tasa excedido |
| 500 | Error interno |

---

## Variables de entorno

```bash
# Servidor
EPISTEME_API_HOST=0.0.0.0
EPISTEME_API_PORT=8000
EPISTEME_API_KEYS=key1,key2

# Datos
EPISTEME_DATA_DIR=~/.episteme/data
EPISTEME_DB_PATH=~/.episteme/db/episteme.db

# Registro
RUST_LOG=info
```

---

## Licencia

Licencia APACHE-2.0
