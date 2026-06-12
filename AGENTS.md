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
- MCP: Unified `Transport` enum (HTTP/stdio) — install 시 선택한 transport가 모든 AI 도구(Claude, Cursor, Gemini, OpenCode, Cline)에 동일하게 시딩됨; `Transport` variants carry an optional `token` field for bearer token auth (non-localhost binding requires token in strict mode)
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

## Evaluation
- Runner: `python3 benchmarks/eval_runner.py full` (requires Python ≥ 3.12, `cargo build` first)
- Suites: `search-positive`, `search-negative`, `smell-negative`, `analyze-positive`, `traversal`, `full`
- Composite: `0.3*recall + 0.3*precision + 0.2*specificity + 0.2*smell_recall`
- Regression: fails if composite drops ≥ 0.02 or any metric drops ≥ 0.05
- CI: `.github/workflows/eval.yml` runs on PRs touching `src/`, `meta/`, `benchmarks/`
- Test sets: `benchmarks/{search_eval_set,search_negative_eval_set,analyze_eval_set,traversal_eval_set}.json` + `smell_negative_corpus/`
- Dashboard: `benchmarks/dashboard/` (Svelte 5, `npm run dev`)
- Docs: `docs/evaluation.md`
- Unit tests: `python3 -m pytest benchmarks/test_eval_runner.py`

## Testing
- Framework: Built-in `#[test]` + `proptest` | Run all: `cargo test` | Coverage: N/A
- File naming: Inline `mod tests` within each source file
- Mocking: Concrete test data (JSON fixtures in `raw/`), `tempfile` for FS isolation

## Version Bump Checklist

버전 범프 시 반드시 아래 파일 모두 동일 버전으로 변경:

| 파일 | 위치 |
|------|------|
| `Cargo.toml` | `version = "x.y.z"` |
| `web/package.json` | `"version": "x.y.z"` |
| `web/src-tauri/Cargo.toml` | `version = "x.y.z"` |
| `web/src-tauri/tauri.conf.json` | `"version": "x.y.z"` |
| `.claude-plugin/plugin.json` | `"version": "x.y.z"` |
| `.codex-plugin/plugin.json` | `"version": "x.y.z"` |

`src/server/mcp_schemas.rs`의 `SERVER_VERSION`은 `env!("CARGO_PKG_VERSION")`을 사용하므로 자동 동기화됨.

아래는 **건드리지 않음** (의도적으로 분리된 버전):
- `meta/schema.json` — 지식 그래프 스키마 버전
- `benchmarks/dashboard/package.json` — 벤치마크 대시보드
- `docs/` — 문서 내 버전 참조

## Git Workflow
- Branch strategy: Feature branches (`feature/<name>`)
- Commit format: Conventional Commits (`type(scope): description`)
- CI: GitHub Actions — release workflow on `v*.*.*` tags; cross-compiles for linux-x86_64, linux-aarch64, macos-x86_64, macos-aarch64
- PR requirements: Pass `cargo clippy -- -D warnings` + `cargo test`

## Boundaries
- Always: Run `cargo clippy -- -D warnings` and `cargo test` before committing changes
- Always: Run `python3 benchmarks/eval_runner.py full` before merging changes to `src/` or `meta/` — check for regression
- Always: Keep `domain/` free of external crate dependencies (no thiserror, no serde in graph.rs)
- Always: Implement new parsers via the `CodeParser` trait in `ports/parser.rs`
- Always: Use `OnceLock<Regex>` for regex caching — never compile patterns in hot loops
- Always: Add `#[cfg(test)] mod tests` at the bottom of the file you modified
- Always: Use entity IDs (`DP-xxx`, `RF-xxx`, `LAW-xxx`, `SMELL-xxx`) when referencing knowledge graph entries
- Ask first: Adding new dependencies to `Cargo.toml`
- Ask first: Adding new test cases to eval sets — must follow existing schema and naming conventions
- Ask first: Changing `ports/` trait signatures (affects all adapters)
- Ask first: Modifying `meta/relations.json` — `solved_by` must NOT be stored (derived at load time from `solves`); `meta/schema.json` (knowledge graph schema)
- Never: Import adapter types from `domain/` — depend on port traits instead
- Never: Use `unwrap()` in production code — use `?`, `ok_or`, or proper error handling
- Never: Use `#[allow(...)]` to suppress warnings — fix the root cause; use `#[expect(...)]` only with a tracking issue comment
- Never: Modify `dist/` or `db/` directly — use `episteme dist` and `episteme build` commands
- Never: Commit `target/` or `.env` files
- Never: Delete or modify eval test sets (`benchmarks/*_eval_set.json`, `smell_negative_corpus/`) without running `eval_runner.py full` before and after
