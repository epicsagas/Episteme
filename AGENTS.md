# AGENTS.md

## Commands
- Build: `cargo build` | Build (release): `cargo build --release`
- Test: `cargo test` | Test single: `cargo test <test_name>` | Test module: `cargo test --lib <module>::tests`
- Lint: `cargo clippy -- -D warnings` | Format: `cargo fmt` | Run: `cargo run -- <subcommand>`
- Gen/Sync: N/A

## Project Structure
- `src/main.rs` — CLI entry point (clap derive, 13 subcommands)
- `src/lib.rs` — Library root, re-exports primary types
- `src/cli.rs` — Clap command enum definitions
- `src/commands/` — CLI subcommand handlers: `analysis`, `build`, `search`, `graph`, `install`, `service`, `other`
- `src/domain/` — Business logic (no external deps): `graph` (KnowledgeGraph, BFS, inverse relation derivation), `detectors` (16 smell detectors), `engine` (RefactoringRanker), `inference`, `summarizer`, `problem_mapper`, `types` (Entity, EntityType, RelationType, SmellType), `metrics` (CodeMetrics, SmellDetection)
- `src/ports/` — Traits (hexagonal boundaries): `parser` (CodeParser), `search` (SearchIndex), `graph` (GraphRepository), `embeddings` (EmbeddingProvider)
- `src/adapters/` — Infrastructure: `regex_parsers` (GenericParser, 10 langs), `python_ast_parser` (rustpython-parser), `search_engines` (FTS5 + cosine), `sqlite_db`, `cache`, `local_embeddings` (fastembed/ONNX), `openai_embeddings`, `service` (MCP HTTP daemon), `installer` (Transport enum, upsert_dir, multi-tool seeding), `install_wizard` (TUI transport selection), `config`, `metrics`, `telemetry`, `paths`, `hooks`, `json_loader`, `constants`, `error`, `rate_limiter`, `structured_logging`, `chunker`, `builder`, `rate_limiter_mw`
- `src/server/` — HTTP layer (axum): `api_routes` (REST endpoints), `api_server`, `api_app`, `api_middleware`, `api_models`, `mcp_handler` (MCP facade), `mcp_dispatcher`, `mcp_search`, `mcp_graph`, `mcp_analysis`, `mcp_schemas`, `mcp_transport_http`, `mcp_auth`, `web_viewer` (D3-force graph UI)
- `raw/` — Knowledge base source data: `code-smells/`, `design-patterns/`, `refactoring/`, `software-engineering/`
- `meta/` — Generated metadata: `relations.json` (solves is SSOT; solved_by derived at load time), `taxonomy.json`, `schema.json`, `code_smells.json`
- `registry/` — Seedable artifacts: `agents/` (4 AI agent definitions), `skills/episteme/` (skill definition)
- `docs/` — User documentation: `api.md`, `mcp-integration-guide.md`, `distribution.md`, `alcove-integration.md`
- `benchmarks/` — Search quality benchmarks and evaluation
- `monitoring/` — Prometheus + Grafana + Alertmanager config
- `db/` — Pre-built SQLite database (`episteme.db`)
- `dist/` — Release packaging artifacts

## Code Style
- Rust edition 2024, MSRV 1.82+
- Naming: `snake_case` functions/vars, `PascalCase` types/enums, `SCREAMING_SNAKE` constants, `kebab-case` file names
- Error handling: `thiserror` for domain errors (`GraphError`), `InfraError` for adapters; `anyhow::Result` in CLI/main; domain uses manual `Display` + `Error` impls (no thiserror dep)
- Imports: `use crate::domain::...` for domain, `use crate::ports::...` for traits, `use crate::adapters::...` for infra; grouped by source
- Async: `tokio` runtime; axum handlers are async; domain logic is synchronous
- MCP: Unified `Transport` enum (HTTP/stdio) — install 시 선택한 transport가 모든 AI 도구(Claude, Cursor, Gemini, OpenCode, Cline)에 동일하게 시딩됨
- State: `KnowledgeGraph` is the central in-memory data structure; `EpistemeMCP` wraps it with optional RAG; `solves`가 단일 진실 공급원, `solved_by`는 `derive_inverse_relations()`에서 로드 시 파생
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
- Ask first: Modifying `meta/relations.json` — `solved_by` must NOT be stored (derived at load time from `solves`); `meta/schema.json` (knowledge graph schema)
- Never: Import adapter types from `domain/` — depend on port traits instead
- Never: Use `unwrap()` in production code — use `?`, `ok_or`, or proper error handling
- Never: Use `#[allow(...)]` to suppress warnings — fix the root cause; use `#[expect(...)]` only with a tracking issue comment
- Never: Modify `dist/` or `db/` directly — use `episteme dist` and `episteme build` commands
- Never: Commit `target/` or `.env` files
