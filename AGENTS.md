# AGENTS.md

## Commands
- Build: `cargo build` | Build (release): `cargo build --release`
- Test: `cargo test` | Test single: `cargo test <test_name>` | Test module: `cargo test --lib <module>::tests`
- Lint: `cargo clippy -- -D warnings` | Format: `cargo fmt` | Run: `cargo run -- <subcommand>`
- Gen/Sync: N/A

## Project Structure
- `src/main.rs` — CLI entry point (clap derive, 15 subcommands)
- `src/bin/syntagma-mcp.rs` — Standalone MCP stdio binary (JSON-RPC over stdin/stdout)
- `src/lib.rs` — Library root, re-exports primary types
- `src/cli.rs` — Clap command enum definitions
- `src/commands/` — CLI subcommand handlers (`analysis`, `build`, `explore`, `graph`, `install`, `service`, `other`)
- `src/domain/` — Business logic (no external deps): `graph` (KnowledgeGraph, BFS), `detectors` (16 smell detectors), `engine` (RefactoringRanker), `summarizer`, `problem_mapper`, `types` (Entity, EntityType, RelationType, SmellType), `metrics` (CodeMetrics, SmellDetection)
- `src/ports/` — Traits (hexagonal boundaries): `parser` (CodeParser), `search` (SearchIndex), `graph` (GraphRepository), `embeddings` (EmbeddingProvider)
- `src/adapters/` — Infrastructure: `regex_parsers` (GenericParser, 10 langs), `python_ast_parser` (rustpython-parser), `search_engines` (FTS5 + cosine), `sqlite_db`, `cache` (Redis optional), `local_embeddings` (fastembed/ONNX), `openai_embeddings`, `service` (MCP HTTP daemon), `installer`, `config`, `metrics`, `telemetry`
- `src/server/` — HTTP layer (axum): `api_routes` (17 REST endpoints), `mcp_handler` (MCP facade), `mcp_dispatcher`, `mcp_search`, `mcp_graph`, `mcp_analysis`, `mcp_schemas`, `mcp_transport_http`, `web_viewer` (D3-force graph UI), `api_app`, `api_middleware`, `api_models`, `mcp_auth`
- `raw/` — Knowledge base source data: `code-smells/`, `design-patterns/`, `refactoring/`, `software-engineering/`
- `meta/` — Generated metadata: `relations.json`, `taxonomy.json`, `schema.json`, `code_smells.json`
- `docs/` — User documentation: `api.md`, `mcp-integration-guide.md`, `distribution.md`, `alcove-integration.md`
- `benchmarks/` — Search quality benchmarks and evaluation
- `monitoring/` — Prometheus + Grafana + Alertmanager config
- `db/` — Pre-built SQLite database (`syntagma.db`)
- `dist/` — Release packaging artifacts

## Code Style
- Rust edition 2024, MSRV 1.82+
- Naming: `snake_case` functions/vars, `PascalCase` types/enums, `SCREAMING_SNAKE` constants, `kebab-case` file names
- Error handling: `thiserror` for domain errors (`GraphError`), `InfraError` for adapters; `anyhow::Result` in CLI/main; domain uses manual `Display` + `Error` impls (no thiserror dep)
- Imports: `use crate::domain::...` for domain, `use crate::ports::...` for traits, `use crate::adapters::...` for infra; grouped by source
- Async: `tokio` runtime; axum handlers are async; domain logic is synchronous
- State: `KnowledgeGraph` is the central in-memory data structure; `SyntagmaMCP` wraps it with optional RAG
- Regex: `OnceLock<Regex>` for cached static patterns; `GenericParser` with `ParserConfig` for language-specific parsers
- Architecture: Hexagonal (ports & adapters) — `domain/` has zero external deps, `ports/` defines traits, `adapters/` implements them
- Tests: Inline `#[cfg(test)] mod tests` at bottom of each file; `tempfile` for FS tests, `proptest` for property tests

### Golden Path
```rust
// Domain: smell detection with tiered confidence accumulation
use crate::domain::metrics::{CodeMetrics, SmellDetection};

fn detect_long_method(metrics: &CodeMetrics, loc: &str, name: &str) -> Option<SmellDetection> {
    let mut acc = TieredAccum::new();
    acc.tier(metrics.loc, 50, 0.30, format!("LOC={}", metrics.loc), 30, 0.15, format!("LOC={}", metrics.loc));
    acc.tier(metrics.cyclomatic_complexity, 15, 0.40, "CC>15".into(), 10, 0.25, "CC>10".into());
    acc.into_detection("SMELL-01", "Long Method", loc, name, metrics, 0.30)
}
```

## Testing
- Framework: Built-in `#[test]` + `proptest` | Run all: `cargo test` | Coverage: N/A
- File naming: Inline `mod tests` within each source file
- Mocking: Concrete test data (JSON fixtures in `raw/`), `tempfile` for FS isolation

## Git Workflow
- Branch strategy: Feature branches (`feature/<name>`)
- Commit format: Conventional Commits (`type(scope): description`)
- CI: GitHub Actions — release workflow on `v*.*.*` tags; cross-compiles for linux-x86_64, linux-aarch64, macos-x86_64, macos-aarch64
- PR requirements: Pass `cargo clippy -- -D warnings` + `cargo test`

## Boundaries
- Always: Run `cargo clippy -- -D warnings` and `cargo test` before committing changes
- Always: Keep `domain/` free of external crate dependencies (no thiserror, no serde in graph.rs)
- Always: Implement new parsers via the `CodeParser` trait in `ports/parser.rs`
- Always: Use `OnceLock<Regex>` for regex caching — never compile patterns in hot loops
- Always: Add `#[cfg(test)] mod tests` at the bottom of the file you modified
- Always: Use entity IDs (`DP-xxx`, `RF-xxx`, `LAW-xxx`, `SMELL-xxx`) when referencing knowledge graph entries
- Ask first: Adding new dependencies to `Cargo.toml`
- Ask first: Changing `ports/` trait signatures (affects all adapters)
- Ask first: Modifying `meta/relations.json` or `meta/schema.json` (knowledge graph schema)
- Never: Import adapter types from `domain/` — depend on port traits instead
- Never: Use `unwrap()` in production code — use `?`, `ok_or`, or proper error handling
- Never: Use `#[allow(...)]` to suppress warnings — fix the root cause; use `#[expect(...)]` only with a tracking issue comment
- Never: Modify `dist/` or `db/` directly — use `syntagma dist` and `syntagma build` commands
- Never: Commit `target/` or `.env` files
