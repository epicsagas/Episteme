# Changelog

All notable changes to Episteme will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **MCP HTTP transport for Claude Code** — transport selector TUI, HTTP as default, launchd auto-enable
- **Agent prompt auto-install** — `epis install` copies Episteme agent prompts into `~/.claude/agents/`
- **Entity descriptions** — description field auto-extracted from markdown source files, shown in web viewer detail panel
- **Benchmark visualization SPA** — trend analysis, query breakdown dashboard
- **Web viewer redesign** — Sankey diagram layout, sidebar tree, detail panel, subgraph readability improvements
- **MCP config upsert** — re-running `epis install` updates transport when config differs (stdio ↔ HTTP)
- **MCP yaml config** — `mcp.host` / `mcp.port` in `config.yaml` (yaml → env fallback)
- **Monitoring** — native and remote Prometheus scrape target support via env
- **CI hardening** — cargo audit, gitleaks, SBOM generation, pinned action SHAs
- **Release pipeline** — Windows target, crates.io publishing, Homebrew tap
- **God module architectural diagnosis example** in `examples/`

### Changed

- **Install wizard** — all steps (transport, Redis, telemetry) migrated to fullscreen TUI
- **Install flow** — auto-builds RAG index after seeding, skips when DB already exists
- **Knowledge graph** — enriched with cross-entity semantic relations
- **License** — MIT → Apache-2.0

### Fixed

- Tokio runtime panic in synchronous `main()` for telemetry
- Search quality — NDCG measurement bug resolved, hit@1 accuracy improved to 100%
- Search recall — cross-type boosting, sparse entity handling, intent synonyms
- fastembed model cache pinned to `~/.episteme/models`
- launchd bootstrap UID substitution and port-in-use handling
- CORS origins now configurable via `EPISTEME_CORS_ORIGINS`

## [0.1.0] - 2026-05-03

### Added

- **Full Rust rewrite** — complete replacement of Python codebase with idiomatic Rust
- **Hexagonal architecture** — `ports/` (traits), `domain/` (business logic), `adapters/` (infrastructure), `server/` (HTTP)
- **GenericParser framework** — 8 brace-based parsers consolidated into `GenericParser` with `ParserConfig`; regex patterns cached via `OnceLock` with `Box::leak`
- **Python AST parsing** — `rustpython-parser` for accurate Python smell detection (Long Method, Large Class, God Object)
- **TieredAccum + build_detection()** — deduplicated 14 identical smell detection constructions in `detectors.rs` (1,253 → 591 lines)
- **MCP module decomposition** — split `EpistemeMCP` (675 lines) into `mcp_search`, `mcp_graph`, `mcp_analysis` services
- **CLI command decomposition** — split `main.rs` (1,741 lines) into `commands/` module with `cli.rs` for clap definitions
- **API handler deduplication** — merged duplicate `search`/`search_post` into shared `do_search()`
- **16 smell detector functions** — up from 14, covering all GoF smell categories
- **17 REST API endpoints** — health probes, Prometheus metrics, CORS, rate limiting
- **Rate limiter TTL eviction** — MAX_BUCKETS=10,000 with 1-hour TTL to prevent unbounded memory growth
- **ReDoS mitigation** — bounded ternary operator regex from `[^:]+` to `[^:\n]{1,50}`
- **Local embeddings** — fastembed (ONNX Runtime) for zero-config semantic search
- **Interactive install wizard** — TUI with crossterm, vim keybindings, alternate screen
- **Distribution packaging** — `episteme dist` command for release archive creation with auto DB bootstrap
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
- `episteme-hook` standalone binary (was Python-only PyPI entry point) — use `episteme hooks ground|sniff|audit` instead

## [0.0.5] - 2026-04-30

### Added

- Graph visualization web UI (`episteme web`) with D3-force
- Pre-built vector DB in release archive
- `epis install --local` flag for development workflows
- 650+ semantic relations covering all 161 entities
- CI auto-generate vector DB during release

## [0.0.4] - 2026-04-29

### Added

- MCP server with 6 tools
- 4 specialized agents
- `epis install` command
- `epis service` daemon management
- Hybrid search (FTS5 + vector)
- Redis caching, GPU acceleration
- 10-language code smell detection
- Prometheus + Grafana monitoring
