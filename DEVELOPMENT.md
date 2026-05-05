# Syntagma Development Guide

**Project:** Syntagma v0.1.0
**Language:** Rust (edition 2024)
**Last Updated:** 2026-05-03

---

## Current Status

| Component | Status | Details |
|-----------|--------|---------|
| **Knowledge Base** | Complete | 22 patterns, 66 refactorings, 56 laws, 23 smells, 201 relations |
| **Code Smell Detection** | Production | 16 detector functions, 10 languages |
| **REST API** | Production | 17 endpoints (axum), rate limiting, auth |
| **MCP Server** | Production | 6 tools, stdio + HTTP transport |
| **RAG Pipeline** | Production | SQLite + FTS5 + fastembed (ONNX) |
| **Graph Visualization** | Production | Interactive web UI with D3-force |

---

## Architecture

Hexagonal (ports & adapters) architecture:

```
src/
├── commands/          # CLI subcommand handlers (clap)
│   ├── analysis.rs    # analyze, infer
│   ├── build.rs       # build (RAG pipeline)
│   ├── explore.rs     # explore (search/REPL)
│   ├── graph.rs       # graph queries
│   ├── install.rs     # install wizard (TUI)
│   ├── service.rs     # MCP HTTP daemon management
│   └── other.rs       # api, mcp, web, telemetry, hooks
├── adapters/          # Infrastructure layer
│   ├── regex_parsers.rs   # GenericParser (10 languages, OnceLock regex cache)
│   ├── python_ast_parser.rs  # Python AST (rustpython-parser)
│   ├── search_engines.rs  # FTS5 keyword + cosine similarity
│   ├── service.rs         # MCP HTTP daemon
│   ├── sqlite_db.rs       # SQLite connection pool
│   ├── cache.rs           # Redis caching (optional)
│   └── ...
├── domain/            # Business logic (no external deps)
│   ├── graph.rs       # KnowledgeGraph (BFS, subgraph, contradictions, Jaccard)
│   ├── detectors.rs   # 16 smell detectors with TieredAccum
│   ├── engine.rs      # RefactoringInferenceEngine + RefactoringRanker
│   ├── summarizer.rs  # Detail-level response optimization
│   └── types.rs       # EntityType, RelationType, core types
├── server/            # HTTP layer (axum)
│   ├── api_routes.rs  # 17 REST endpoints
│   ├── mcp_handler.rs # MCP thin facade
│   ├── mcp_search.rs  # Search service
│   ├── mcp_graph.rs   # Graph service
│   └── mcp_analysis.rs # Code analysis service
└── ports/             # Traits (hexagonal boundaries)
    ├── parser.rs      # CodeParser trait
    ├── search.rs      # SearchEngine trait
    ├── graph.rs       # GraphStore trait
    └── embeddings.rs  # EmbeddingProvider trait
```

---

## Tech Stack

| Component | Technology | Purpose |
|-----------|-----------|---------|
| **Language** | Rust (edition 2024) | Safety, performance, single-binary |
| **Web Framework** | axum | REST API + MCP HTTP transport |
| **Database** | rusqlite (bundled SQLite) | Knowledge graph + vector store |
| **Search** | FTS5 + cosine similarity | Keyword + semantic hybrid search |
| **Embeddings** | fastembed (ONNX Runtime) | Local, zero-config embedding generation |
| **CLI** | clap (derive) | 15 subcommands |
| **Python AST** | rustpython-parser | AST-based Python smell detection |
| **Other Languages** | regex (OnceLock cached) | GenericParser framework |

---

## Code Smell Detectors (16)

| ID | Smell | Detection |
|----|-------|-----------|
| SMELL-01 | Long Method | LOC threshold |
| SMELL-02 | Long Parameter List | Parameter count |
| SMELL-03 | Primitive Obsession | Primitive parameter ratio |
| SMELL-04 | Large Class | Method + field count |
| SMELL-05 | Data Clumps | Repeated parameter groups (stub) |
| SMELL-06 | Switch Statements | Switch/match count |
| SMELL-07 | Data Class | Methods vs fields ratio |
| SMELL-08 | Temporary Field | Conditional field usage (stub) |
| SMELL-09 | Shotgun Surgery | Change coupling (stub) |
| SMELL-10 | Divergent Change | Method cohesion metrics |
| SMELL-11 | Lazy Class | Low LOC + method count |
| SMELL-12 | Speculative Generality | Abstract without concrete |
| SMELL-13 | Duplicate Code | Hash-based similarity (partial) |
| SMELL-14 | Middle Man | Delegation ratio |
| SMELL-15 | Parallel Inheritance Hierarchies | Hierarchy mirroring (stub) |
| SMELL-16 | Comments | Comment-to-code ratio (stub) |
| SMELL-17 | Dead Code | Unreachable/unused detection (stub) |
| SMELL-18 | Feature Envy | External call ratio |
| SMELL-19 | Inappropriate Intimacy | Cross-class private access (stub) |
| SMELL-20 | Message Chains | Call chain depth |
| SMELL-21 | God Object | Composite: LOC + methods + coupling |
| SMELL-22 | Refused Bequest | Override-to-nothing ratio (stub) |
| SMELL-23 | Alternative Classes with Different Interfaces | Interface divergence (stub) |

---

## Development Setup

```bash
# Clone and build (requires Rust 1.85+)
git clone https://github.com/epicsagas/Syntagma.git
cd Syntagma
cargo build

# Run tests
cargo test

# Lint
cargo clippy -- -D warnings

# Install locally (seeds data and builds DB automatically)
cargo install --path .
syntagma install --local
```

---

## API Endpoints (17)

| Method | Path | Description |
|--------|------|-------------|
| GET | `/` | Service info |
| GET | `/health` | Health check |
| GET | `/live` | Liveness probe |
| GET | `/ready` | Readiness probe |
| GET | `/stats` | Graph statistics |
| POST | `/analyze` | Code smell detection |
| POST | `/refactor` | Refactoring suggestions |
| GET | `/search` | Knowledge search |
| POST | `/search` | Knowledge search (POST) |
| GET | `/graph/{id}` | Get entity |
| GET | `/graph/{id}/neighbors` | Get neighbors |
| POST | `/graph/neighbors` | Get neighbors (POST) |
| POST | `/graph/subgraph` | Extract subgraph |
| GET | `/graph/path` | Shortest path |
| GET | `/graph/contradictions` | Find contradictions |
| POST | `/graph/infer-transitive` | Infer transitive relations |
| GET | `/metrics` | Prometheus metrics |

---

## Future Roadmap

- **IDE Plugins** — VSCode, IntelliJ native integrations
- **Custom Entities** — Add team-specific patterns/smells
- **Team Metrics** — Aggregate pattern usage across organization
- **Multilingual Docs** — Knowledge base in Korean, Japanese, Chinese
- **Interactive Tutorials** - In-app guided tours for MCP tools

---

*Last Updated: 2026-05-03*
