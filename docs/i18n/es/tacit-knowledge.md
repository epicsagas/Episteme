# Arquitectura del conocimiento tacito

Episteme gestiona dos capas distintas de conocimiento: **canonico** (inmutable, curado) y **tacito** (mutable, contribuido por usuarios). Este documento describe la arquitectura de dos bases de datos, el flujo de datos y el ciclo de vida de los insights.

## Vista general

| | Conocimiento canonico | Conocimiento tacito (Insights) |
|---|---|---|
| **Almacenamiento** | `~/.episteme/db/episteme.db` | `~/.episteme/user_knowledge.db` |
| **Mutabilidad** | Solo lectura (reconstruido via `epis build`) | Lectura-escritura (tiempo real via MCP) |
| **Prefijo de ID** | `DP-NNN`, `RF-NNN`, `LAW-NNN`, `SMELL-NNN` | `TK-NNN` |
| **Fuente** | Archivos markdown curados en `raw/` | Herramienta MCP `add_insight` / CLI `epis insight` |
| **Entidades** | 22 patrones, 66 refactorizaciones, 56 leyes, 23 smells | Insights de usuario ilimitados |

Estas dos bases de datos estan fisicamente separadas pero se fusionan en tiempo de ejecucion en un unico grafo transitable.

## Diseno de dos bases de datos

```
┌─────────────────────────────────┐     ┌──────────────────────────────┐
│  Base canonica (episteme.db)    │     │  Base de conocimiento        │
│                                 │     │  de usuario                  │
│  ┌───────────┐  ┌────────────┐  │     │  (user_knowledge.db)         │
│  │  chunks   │  │ embeddings │  │     │  ┌────────────────────────┐  │
│  │  (914)    │  │  (914)     │  │     │  │  user_entities         │  │
│  └───────────┘  └────────────┘  │     │  │  (entradas TK-xxx)     │  │
│                                 │     │  ├────────────────────────┤  │
│  Construido por: epis build     │     │  │  user_relations        │  │
│  Poblado desde: raw/*.md        │     │  ├────────────────────────┤  │
│                                 │     │  │  user_embeddings       │  │
│  Inmutable en tiempo de         │     │  ├────────────────────────┤  │
│  ejecucion                      │     │  │  user_entities_fts     │  │
│                                 │     │  │  (indice de busqueda   │  │
└──────────────┬──────────────────┘     │  │   FTS5)                │  │
               │                        │  ├────────────────────────┤  │
               │                        │  │  insight_seq           │  │
               │                        │  │  (contador ID atomico) │  │
               │                        │  └────────────────────────┘  │
               │                        │                              │
               │                        │  Escrito por: MCP add_insight│
               │                        │  Leido por: search_insights  │
               │                        └──────────────┬───────────────┘
               │                                       │
               └───────────────┬───────────────────────┘
                               │
                    ┌──────────▼──────────┐
                    │   CompositeGraph    │
                    │   (fusion en        │
                    │    memoria)         │
                    │                     │
                    │  - Busqueda de      │
                    │    entidad unificada│
                    │  - BFS inter-capa   │
                    │  - Consultas de     │
                    │    vecinos inter-   │
                    │    capas            │
                    │                     │
                    │  Sirve todas las    │
                    │    solicitudes de   │
                    │    herramientas MCP │
                    └─────────────────────┘
```

### ¿Por que bases de datos separadas?

1. **Proteccion** — La entrada del usuario no puede corromper el conocimiento canonico curado.
2. **Ciclo de vida independiente** — El conocimiento canonico se actualiza via el pipeline de build; el conocimiento tacito se actualiza en tiempo real.
3. **Portabilidad** — Comparta `user_knowledge.db` entre maquinas o equipos sin tocar la capa canonica.

## CompositeGraph

La estructura `CompositeGraph` (en `src/domain/composite_graph.rs`) fusiona ambas capas en una unica interfaz `GraphRepository` al inicio:

- Carga el `KnowledgeGraph` canonico desde `relations.json`
- Abre `user_knowledge.db` via `UserGraphStore`
- Proporciona metodos `get_entity()`, `get_neighbors()`, `find_path()` unificados a traves de ambas capas
- Las operaciones de usuario nunca modifican el grafo canonico

### Fallback elegante

Si `user_knowledge.db` no puede abrirse (archivo faltante, error de permiso), el sistema recurre al modo solo canonico. Las 6 herramientas MCP canonicas continuan funcionando; las 3 herramientas de conocimiento tacito retornan un error.

## Schema de conocimiento de usuario

```sql
-- Tabla de entidades principal
CREATE TABLE user_entities (
    id TEXT PRIMARY KEY,                    -- ej: "TK-001"
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    author TEXT NOT NULL DEFAULT 'user',
    confidence REAL NOT NULL DEFAULT 0.5,   -- 0.0 a 1.0
    evidence_count INTEGER NOT NULL DEFAULT 0,
    last_validated TEXT NOT NULL DEFAULT '',
    tags TEXT NOT NULL DEFAULT '[]',        -- arreglo JSON
    relations TEXT NOT NULL DEFAULT '{}',   -- JSON: tipo -> [target_ids]
    created_at TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL DEFAULT '',
    link_provenance TEXT NOT NULL DEFAULT '{}'  -- JSON: entity_id -> metadatos
);

-- Aristas de relaciones explicitas
CREATE TABLE user_relations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_id TEXT NOT NULL,
    relation_type TEXT NOT NULL,
    to_id TEXT NOT NULL,
    UNIQUE(from_id, relation_type, to_id)
);

-- Vectores de embeddings (f32, little-endian)
CREATE TABLE user_embeddings (
    entity_id TEXT PRIMARY KEY,
    embedding BLOB NOT NULL
);

-- Indice de busqueda en texto completo
CREATE VIRTUAL TABLE user_entities_fts USING fts5(
    title, content, tags,
    content=user_entities, content_rowid=rowid
);

-- Secuencia ID atomica
CREATE TABLE insight_seq (key TEXT PRIMARY KEY, val INTEGER NOT NULL);
```

## Herramientas MCP

### add_insight

Crea una entidad `TK-NNN` a partir de texto libre. El sistema automaticamente:

1. **Detecta enlaces a entidades canonicas** — Correspondencia de palabras clave en dos fases (filtrado de palabras vacias + scoring compuesto) encuentra patrones, leyes y smells relevantes.
2. **Verifica duplicados** — Compara con insights existentes.
3. **Crea relaciones `derives_from`** — Para enlaces de alta confianza (score >= 0.5), enlaza automaticamente a entidades canonicas.
4. **Computa correlaciones** — Encuentra insights relacionados usando similitud Jaccard.

Parametros:
- `text` (requerido) — Contenido del insight en texto libre
- `project` (opcional) — Tag de nombre de proyecto
- `tags` (opcional) — Tags de categoria
- `linked_entities` (opcional) — IDs de entidades explicitas para enlazar (ej: `["DP-005", "SMELL-01"]`)

### search_insights

Busqueda por palabras clave FTS5 en insights contribuidos por usuarios. Retorna entidades `TK-*` coincidentes con su contenido y relaciones.

Parametros:
- `query` (requerido) — Consulta de busqueda en lenguaje natural
- `limit` (opcional) — Maximo de resultados (por defecto 10, maximo 20)

### confirm_links

Valida o rechaza enlaces detectados automaticamente entre un insight y entidades canonicas. Cada confirmacion:

- Incrementa el score de confianza del insight (+0.05 por enlace confirmado, limite 1.0)
- Registra la procedencia del enlace (fuente, score, marca de tiempo)
- Soporta relaciones merge/supersede entre insights

Parametros:
- `insight_id` (requerido) — El ID `TK-NNN`
- `accepted` (requerido) — IDs de entidades a confirmar como enlaces validos
- `rejected` (opcional) — IDs de entidades a rechazar
- `merged_with` (opcional) — ID del insight objetivo para fusion/reemplazo

## Ciclo de vida de un insight

```
1. add_insight("마이크로서비스 분리 시 도메인 경계를 먼저 식별하기로 결정")
       │
       ▼
2. Deteccion automatica de enlaces: CONWAY-001 (Ley de Conway), DP-026 (Strangler Fig)
       │
       ▼
3. Crear TK-001 con derives_from → LAW-017, DP-026
       │
       ▼
4. confirm_links(insight_id="TK-001", accepted=["LAW-017"])
       │
       ▼
5. Confianza incrementada: 0.5 → 0.55
       │
       ▼
6. Mas tarde: search_insights("마이크로서비스 분리") → retorna TK-001
       │
       ▼
7. find_path("TK-001", "SMELL-03") → recorre el grafo inter-capas
```

## Tipos de relaciones

| Relacion | Direccion | Descripcion |
|----------|-----------|-------------|
| `derives_from` | TK → Canonico | Insight fundamentado en una entidad canonica |
| `applies_to` | TK → Canonico | Insight que aplica un patron/ley a un contexto especifico |
| `supersedes` | TK → TK | Un insight mas reciente reemplaza uno anterior |
| `related_to` | TK → TK/Canonico | Conexion semantica general |

## Uso via CLI

```bash
# Agregar un insight
epis insight add "팀에서 God Class 리팩토링 시 Extract Class보다 Facade Pattern이 효과적이었음"

# Buscar insights
epis insight search "인증 미들웨어"

# Listar todos los insights
epis insight list
```

## Archivos fuente clave

| Archivo | Rol |
|---------|-----|
| `src/domain/composite_graph.rs` | Fusion en tiempo de ejecucion de capas canonica + usuario |
| `src/adapters/user_graph_store.rs` | `MutableGraphRepository` respaldado por SQLite |
| `src/server/mcp_insight.rs` | Manejadores MCP para las 3 herramientas de conocimiento tacito |
| `src/adapters/insight_utils.rs` | Generacion de IDs, marcas de tiempo, utilidades de texto |
| `src/domain/types.rs` | `UserEntity`, `LinkProvenance`, `EntityType::Insight` |
| `src/ports/graph.rs` | Trait `MutableGraphRepository` (14 metodos) |
