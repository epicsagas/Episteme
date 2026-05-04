# Changelog

All notable changes to Syntagma will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-05-03

### Added

- **Full Rust rewrite** — complete replacement of Python codebase with idiomatic Rust
- **Hexagonal architecture** — `ports/` (traits), `domain/` (business logic), `adapters/` (infrastructure), `server/` (HTTP)
- **GenericParser framework** — 8 brace-based parsers consolidated into `GenericParser` with `ParserConfig`; regex patterns cached via `OnceLock` with `Box::leak`
- **Python AST parsing** — `rustpython-parser` for accurate Python smell detection (Long Method, Large Class, God Object)
- **TieredAccum + build_detection()** — deduplicated 14 identical smell detection constructions in `detectors.rs` (1,253 → 591 lines)
- **MCP module decomposition** — split `SyntagmaMCP` (675 lines) into `mcp_search`, `mcp_graph`, `mcp_analysis` services
- **CLI command decomposition** — split `main.rs` (1,741 lines) into `commands/` module with `cli.rs` for clap definitions
- **API handler deduplication** — merged duplicate `search`/`search_post` into shared `do_search()`
- **16 smell detector functions** — up from 14, covering all GoF smell categories
- **17 REST API endpoints** — health probes, Prometheus metrics, CORS, rate limiting
- **Rate limiter TTL eviction** — MAX_BUCKETS=10,000 with 1-hour TTL to prevent unbounded memory growth
- **ReDoS mitigation** — bounded ternary operator regex from `[^:]+` to `[^:\n]{1,50}`
- **Local embeddings** — fastembed (ONNX Runtime) for zero-config semantic search
- **Interactive install wizard** — TUI with crossterm, vim keybindings, alternate screen
- **Distribution packaging** — `syntagma dist` command for release archive creation with auto DB bootstrap
- **Cross-platform CI** — GitHub Actions release workflow for linux/macOS (x86_64 + aarch64)
- **Multi-stage Dockerfile** — Rust builder + slim Debian runtime

### Changed

- **Language**: Python 3.11+ → Rust (edition 2024)
- **Web framework**: FastAPI → axum
- **Database**: Python sqlite3 → rusqlite (bundled)
- **Embeddings**: sentence-transformers/PyTorch → fastembed/ONNX Runtime
- **CLI**: argparse → clap (derive)
- **All regex patterns cached** — zero recompilation on hot paths via global `REGEX_CACHE`

### Removed

- Python runtime dependency
- ChromaDB dependency
- tree-sitter dependency
- PyPI publishing workflow

## [0.0.5] - 2026-04-30

### Added

- Graph visualization web UI (`syntagma web`) with D3-force
- Pre-built vector DB in release archive
- `syntagma install --local` flag for development workflows
- 650+ semantic relations covering all 161 entities
- CI auto-generate vector DB during release

## [0.0.4] - 2026-04-29

### Added

- MCP server with 6 tools
- 4 specialized agents
- `syntagma install` command
- `syntagma service` daemon management
- Hybrid search (FTS5 + vector)
- Redis caching, GPU acceleration
- 10-language code smell detection
- Prometheus + Grafana monitoring
