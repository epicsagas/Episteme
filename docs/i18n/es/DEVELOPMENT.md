# Guia de desarrollo Episteme

**Proyecto:** Episteme v0.1.0
**Lenguaje:** Rust (edicion 2024)
**Ultima actualizacion:** 2026-05-03

---

## Estado actual

| Componente | Estado | Detalles |
|------------|--------|----------|
| **Base de conocimiento** | Completo | 22 patrones, 66 refactorizaciones, 56 leyes, 23 smells, 201 relaciones |
| **Deteccion de code smells** | Produccion | 16 funciones detectoras, 10 lenguajes |
| **API REST** | Produccion | 17 endpoints (axum), limitacion de tasa, autenticacion |
| **Servidor MCP** | Produccion | 6 herramientas, transporte stdio + HTTP |
| **Pipeline RAG** | Produccion | SQLite + FTS5 + fastembed (ONNX) |
| **Visualizacion de grafo** | Produccion | Interfaz web interactiva con D3-force |

---

## Arquitectura

Arquitectura hexagonal (puertos y adaptadores):

```
src/
├── commands/          # Manejadores de subcomandos CLI (clap)
│   ├── analysis.rs    # analyze, infer
│   ├── build.rs       # build (pipeline RAG)
│   ├── explore.rs     # explore (busqueda/REPL)
│   ├── graph.rs       # consultas de grafo
│   ├── install.rs     # asistente de instalacion (TUI)
│   ├── service.rs     # gestion del daemon MCP HTTP
│   └── other.rs       # api, mcp, web, telemetria, hooks
├── adapters/          # Capa de infraestructura
│   ├── regex_parsers.rs   # GenericParser (10 lenguajes, cache regex OnceLock)
│   ├── python_ast_parser.rs  # AST Python (rustpython-parser)
│   ├── search_engines.rs  # Palabra clave FTS5 + similitud coseno
│   ├── service.rs         # Daemon MCP HTTP
│   ├── sqlite_db.rs       # Pool de conexiones SQLite
│   ├── cache.rs           # Cache Redis (opcional)
│   └── ...
├── domain/            # Logica de negocio (sin dependencias externas)
│   ├── graph.rs       # KnowledgeGraph (BFS, subgrafo, contradicciones, Jaccard)
│   ├── detectors.rs   # 16 detectores de smells con TieredAccum
│   ├── engine.rs      # RefactoringInferenceEngine + RefactoringRanker
│   ├── summarizer.rs  # Optimizacion de respuestas a nivel de detalle
│   └── types.rs       # EntityType, RelationType, tipos fundamentales
├── server/            # Capa HTTP (axum)
│   ├── api_routes.rs  # 17 endpoints REST
│   ├── mcp_handler.rs # Fachada ligera MCP
│   ├── mcp_search.rs  # Servicio de busqueda
│   ├── mcp_graph.rs   # Servicio de grafo
│   └── mcp_analysis.rs # Servicio de analisis de codigo
└── ports/             # Traits (limites hexagonales)
    ├── parser.rs      # Trait CodeParser
    ├── search.rs      # Trait SearchEngine
    ├── graph.rs       # Trait GraphStore
    └── embeddings.rs  # Trait EmbeddingProvider
```

---

## Stack tecnologico

| Componente | Tecnologia | Proposito |
|------------|-----------|----------|
| **Lenguaje** | Rust (edicion 2024) | Seguridad, rendimiento, binario unico |
| **Framework Web** | axum | API REST + transporte MCP HTTP |
| **Base de datos** | rusqlite (SQLite incluido) | Grafo de conocimiento + almacen vectorial |
| **Busqueda** | FTS5 + similitud coseno | Busqueda hibrida por palabras clave + semantica |
| **Embeddings** | fastembed (ONNX Runtime) | Generacion de embeddings locales sin configuracion |
| **CLI** | clap (derive) | 15 subcomandos |
| **AST Python** | rustpython-parser | Deteccion de smells Python basada en AST |
| **Otros lenguajes** | regex (cache OnceLock) | Framework GenericParser |

---

## Detectores de code smells (16)

| ID | Smell | Deteccion |
|----|-------|-----------|
| SMELL-01 | Long Method | Umbral de LOC |
| SMELL-02 | Long Parameter List | Cantidad de parametros |
| SMELL-03 | Primitive Obsession | Ratio de parametros primitivos |
| SMELL-04 | Large Class | Cantidad de metodos + campos |
| SMELL-05 | Data Clumps | Grupos de parametros repetidos (stub) |
| SMELL-06 | Switch Statements | Cantidad de switch/match |
| SMELL-07 | Data Class | Ratio metodos vs campos |
| SMELL-08 | Temporary Field | Uso condicional de campos (stub) |
| SMELL-09 | Shotgun Surgery | Acoplamiento de cambios (stub) |
| SMELL-10 | Divergent Change | Metricas de cohesion de metodos |
| SMELL-11 | Lazy Class | Bajo LOC + cantidad de metodos |
| SMELL-12 | Speculative Generality | Abstraccion sin implementacion concreta |
| SMELL-13 | Duplicate Code | Similitud basada en hash (parcial) |
| SMELL-14 | Middle Man | Ratio de delegacion |
| SMELL-15 | Parallel Inheritance Hierarchies | Reproduccion de jerarquia (stub) |
| SMELL-16 | Comments | Ratio comentarios/codigo (stub) |
| SMELL-17 | Dead Code | Deteccion de codigo inalcanzable/no usado (stub) |
| SMELL-18 | Feature Envy | Ratio de llamadas externas |
| SMELL-19 | Inappropriate Intimacy | Acceso privado inter-clases (stub) |
| SMELL-20 | Message Chains | Profundidad de cadena de llamadas |
| SMELL-21 | God Object | Compuesto: LOC + metodos + acoplamiento |
| SMELL-22 | Refused Bequest | Ratio de sobrescrituras vacias (stub) |
| SMELL-23 | Alternative Classes with Different Interfaces | Divergencia de interfaz (stub) |

---

## Configuracion del desarrollo

```bash
# Clonar y compilar (requiere Rust 1.95+)
git clone https://github.com/epicsagas/Episteme.git
cd Episteme
cargo build

# Ejecutar tests
cargo test

# Linter
cargo clippy -- -D warnings

# Instalar localmente (pobla datos y construye la base automaticamente)
cargo install --path .
epis install --local
```

---

## Endpoints API (17)

| Metodo | Ruta | Descripcion |
|--------|------|-------------|
| GET | `/` | Informacion del servicio |
| GET | `/health` | Verificacion de salud |
| GET | `/live` | Sonda de vivacidad |
| GET | `/ready` | Sonda de disponibilidad |
| GET | `/stats` | Estadisticas del grafo |
| POST | `/analyze` | Deteccion de code smells |
| POST | `/refactor` | Sugerencias de refactoring |
| GET | `/search` | Busqueda en el conocimiento |
| POST | `/search` | Busqueda en el conocimiento (POST) |
| GET | `/graph/{id}` | Obtener entidad |
| GET | `/graph/{id}/neighbors` | Obtener vecinos |
| POST | `/graph/neighbors` | Obtener vecinos (POST) |
| POST | `/graph/subgraph` | Extraer subgrafo |
| GET | `/graph/path` | Camino mas corto |
| GET | `/graph/contradictions` | Encontrar contradicciones |
| POST | `/graph/infer-transitive` | Inferir relaciones transitivas |
| GET | `/metrics` | Metricas Prometheus |

---

## Hoja de ruta futura

- **Plugins IDE** — Integraciones nativas VSCode, IntelliJ
- **Entidades personalizadas** — Agregar patrones/smells especificos del equipo
- **Metricas de equipo** — Agregar uso de patrones en la organizacion
- **Documentacion multilingue** — Base de conocimiento en coreano, japones, chino
- **Tutoriales interactivos** - Visitas guiadas integradas para herramientas MCP

---

*Ultima actualizacion: 2026-05-03*
