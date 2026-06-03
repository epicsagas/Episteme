<p align="center">
<img src="docs/assets/icon.png" alt="Episteme" width="60%" />
</p>

<p align="center"><sub>Episteme (ἐπιστήμη) — Greek for "systematic knowledge, scientific understanding"</sub></p>

<p align="center">An offline-first, single-binary knowledge graph that connects design patterns, refactoring techniques, and software laws through semantic relationships.<br><b>Built for AI agents first</b> — integrate software engineering expertise directly into Claude Code, Cursor, and other MCP-compatible tools.</p>

<p align="center">Written in Rust · Single binary · Fully offline</p>

---

<p align="center">
  <a href="https://github.com/epicsagas/Episteme/stargazers"><img alt="Stars" src="https://img.shields.io/github/stars/epicsagas/Episteme?style=for-the-badge&labelColor=0d1117&color=ffd700&logo=github&logoColor=white" /></a>
  <a href="https://github.com/epicsagas/Episteme/network/members"><img alt="Forks" src="https://img.shields.io/github/forks/epicsagas/Episteme?style=for-the-badge&labelColor=0d1117&color=2ecc71&logo=github&logoColor=white" /></a>
  <a href="https://github.com/epicsagas/Episteme/issues"><img alt="Issues" src="https://img.shields.io/github/issues/epicsagas/Episteme?style=for-the-badge&labelColor=0d1117&color=ff6b6b&logo=github&logoColor=white" /></a>
  <a href="https://github.com/epicsagas/Episteme/commits/main"><img alt="Last commit" src="https://img.shields.io/github/last-commit/epicsagas/Episteme?style=for-the-badge&labelColor=0d1117&color=58a6ff&logo=git&logoColor=white" /></a>
</p>
<p align="center">
  <a href="https://crates.io/crates/episteme"><img alt="Crates.io" src="https://img.shields.io/crates/v/episteme?style=for-the-badge&labelColor=0d1117&color=fc8d62&logo=rust&logoColor=white" /></a>
  <a href="LICENSE"><img alt="License" src="https://img.shields.io/badge/license-Apache--2.0-3fb950?style=for-the-badge&labelColor=0d1117" /></a>
  <a href="https://blog.rust-lang.org/"><img alt="Rust" src="https://img.shields.io/badge/rust-1.95+-d73a49?style=for-the-badge&labelColor=0d1117&logo=rust&logoColor=white" /></a>
  <a href="https://github.com/epicsagas/Episteme/actions"><img alt="CI" src="https://img.shields.io/github/actions/workflow/status/epicsagas/Episteme/ci.yml?style=for-the-badge&labelColor=0d1117&color=e67e22&logo=githubactions&logoColor=white" /></a>
  <a href="https://buymeacoffee.com/epicsaga"><img alt="Buy Me a Coffee" src="https://img.shields.io/badge/buy_me_a_coffee-FFDD00?style=for-the-badge&labelColor=0d1117&logo=buymeacoffee&logoColor=black" /></a>
</p>

<p align="center">
  English |
  <a href="docs/i18n/ja/">日本語</a> |
  <a href="docs/i18n/ko/">한국어</a> |
  <a href="docs/i18n/de/">Deutsch</a> |
  <a href="docs/i18n/fr/">Français</a> |
  <a href="docs/i18n/zh-CN/">简体中文</a> |
  <a href="docs/i18n/zh-TW/">繁體中文</a> |
  <a href="docs/i18n/pt/">Português</a> |
  <a href="docs/i18n/es/">Español</a> |
  <a href="docs/i18n/hi/">हिन्दी</a>
</p>

---

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/assets/features.png">
  <img src="docs/assets/features.png" align="center" width="100%" alt="Episteme Features Overview" />
</picture>

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/assets/demo.gif">
  <img src="docs/assets/demo.gif" align="center" width="100%" alt="Episteme Demo" />
</picture>

---

## Quick Start

### Claude Code

```
/plugin marketplace add epicsagas/plugins
/plugin install episteme@epicsagas
```

The plugin hook installs the `epis` binary automatically. **Before starting a new session**, run this once in your terminal:

```bash
epis install   # download knowledge graph data from GitHub Releases
```

`epis install` seeds the knowledge graph database and starts the HTTP API server on port 58302. Then start a new Claude Code session and you're done.

Updates with `/plugin update episteme@epicsagas`.

### Codex CLI

```bash
codex plugin marketplace add epicsagas/plugins
```

The plugin hook installs the `epis` binary automatically. **Before starting a new session**, run this once in your terminal:

```bash
epis install   # download knowledge graph data from GitHub Releases
```

`epis install` seeds the knowledge graph database and starts the HTTP API server on port 58302. Updates with `codex plugin update episteme@epicsagas`.

### Other tools

```bash
epis install cursor       # Cursor IDE
epis install opencode     # OpenCode
epis install cline        # Cline
epis install --all        # All supported tools
```

### Manual install

| Method | Command |
|--------|---------|
| **Homebrew** | `brew install epicsagas/tap/episteme` |
| **Shell script** | `curl --proto '=https' --tlsv1.2 -LsSf https://github.com/epicsagas/Episteme/releases/latest/download/episteme-installer.sh \| sh` |
| **cargo** | `cargo binstall episteme` ⚡ or `cargo install episteme` |
| **Docker** | See [Option 3](#option-3-docker-no-rust-required) |

### Verify

```bash
epis --version
epis stats
```

Or from inside Claude Code / Codex CLI:

```
/episteme verify
```

### Try it in 30 seconds

**Option A — CLI:** Point it at any file in your project.

```bash
epis analyze src/domain/engine.rs
```

```
✓ 2 smells detected in src/domain/engine.rs

  SMELL-07 (Large Class) — RefactoringRanker, 743 lines
  → RF-018 Extract Class          priority 0.89  effort: medium
  → RF-001 Extract Method         priority 0.76  effort: small
  → Violates: LAW-001 Single Responsibility Principle

  SMELL-01 (Long Method) — rank_refactorings(), 58 lines
  → RF-001 Extract Method         priority 0.92  effort: small
  → Violates: LAW-001 SRP, LAW-004 DRY
```

**Option B — Claude Code:** Open any file in your project and ask naturally.

```
Find code smells in this project and suggest refactorings.
```

Episteme auto-triggers — no special syntax needed. It maps your description to the knowledge graph and returns ranked, citable results.

---

## Why Episteme?

LLMs already know what the Strategy pattern is. They can recite SOLID principles, list GoF patterns, and explain code smells. So why does this project exist?

**The gap isn't knowledge — it's structured, connected reasoning.**

When you ask an LLM "how do I fix a God Object?", it gives you a reasonable answer. But the answer changes between conversations, lacks traceability, and doesn't connect the problem to its root causes or downstream consequences. Episteme turns isolated facts into a traversable graph where every recommendation is grounded, citable, and connected to the broader design landscape.

### How is this different from just prompting an LLM well?

| | Well-crafted LLM prompt | Episteme + LLM |
|---|---|---|
| Proactive detection | Only if the user asks the right question | Auto-triggers on problem descriptions |
| Token efficiency | Long explanations + multiple follow-up turns | One tool call returns structured result |
| Relationship traversal | One-hop at best, often hallucinated | Multi-hop graph traversal, verified |
| Cross-referencing | Manual, error-prone | Automated via 201 semantic relations |
| Consistency | Varies between conversations | Same structured answer every time |
| Citability | "I think you should use Extract Class" | "Extract Class (RF-018), priority 0.89" |
| Offline / Air-gapped | Requires internet for best results | Fully local, single binary |

### When is this useful?

<details>
<summary><b>1. When your AI agent should proactively detect problems, not wait to be asked</b></summary>

The MCP integration auto-triggers on problem descriptions. When a user says "this class does too much", the agent doesn't need to know to ask about God Object — Episteme maps the complaint to `SMELL-03`, surfaces ranked refactorings, and traces the violation back to first principles. This turns a vague complaint into a structured remediation plan.
</details>

<details>
<summary><b>2. When you want to reduce token consumption — not burn it on explanations</b></summary>

Without Episteme, an LLM answers "how do I fix a God Object?" by explaining the smell, listing refactorings, describing SOLID principles, and walking through each option — hundreds of tokens per response. With Episteme, one MCP tool call returns `SMELL-03 → RF-018 (0.89) → LAW-001`. Same expertise at a fraction of the token budget.
</details>

<details>
<summary><b>3. When you need code analysis connected to remediation — not just detection</b></summary>

Tools like SonarQube detect smells. LLMs can suggest patterns. Episteme does both and connects them: detect Long Method → trace to the laws it violates → rank the refactorings that solve it → show what patterns enforce those refactorings.
</details>

<details>
<summary><b>4. When isolated pattern knowledge isn't enough — you need the relationships</b></summary>

Knowing what Extract Method does is table stakes. Knowing that it *solves* Long Method (SMELL-01), which *violates* Single Responsibility (LAW-001), which is *enforced by* Facade Pattern (DP-012) — that's a reasoning chain an LLM can't reliably construct on its own. Episteme's 201 semantic relations let AI agents traverse these paths deterministically.
</details>

<details>
<summary><b>5. When you're making architecture decisions and need evidence, not opinions</b></summary>

"Should I use microservices?" — Episteme connects the question to Conway's Law (LAW-017), SRP (LAW-001), and the Strangler Fig pattern (DP-026), then shows how they relate. Decisions become traceable to engineering laws, not blog posts.
</details>

<details>
<summary><b>6. When you need consistent, citable engineering advice — not hallucinated recommendations</b></summary>

Every finding references explicit entity IDs (`DP-005`, `RF-001`, `LAW-021`). Recommendations come with priority scores and effort estimates. The same query always returns the same structured answer.
</details>

<details>
<summary><b>7. When you're working in an air-gapped or restricted network</b></summary>

Episteme runs entirely offline: single binary, local SQLite database, local embeddings via fastembed (ONNX Runtime). No telemetry, no phone-home, no external API calls. Your code and analysis results never leave your machine.
</details>

---

## Features

| | Feature | Why it matters |
|--|---------|----------------|
| 🧠 | **22 GoF Design Patterns** | Complete catalog with real-world examples |
| 🔧 | **66 Refactoring Techniques** | Fowler's catalog with code samples |
| ⚖️ | **56 Software Laws & Principles** | SOLID, Conway's Law, CAP Theorem, etc. |
| 👃 | **17 Code Smell Types** | Long Method, God Object, Feature Envy, etc. ¹ |
| 🔗 | **201 Semantic Relations** | "solves", "enforces", "violates", "relates_to" |
| 🤖 | **9 MCP Tools + 4 Agents** | High-fidelity AI agent interaction with cross-agent handoffs |
| 🌐 | **HTTP API Server** | REST API on port 58302, auto-started on install |
| 🌍 | **10 Language Support** | Python (AST), Java, TypeScript, Go, Rust, C++, C#, PHP, Ruby, Kotlin |
| 📊 | **Deterministic Analysis** | AST-based Python + regex multi-language, same result every time |
| 🏷️ | **Citable Knowledge** | Every finding links to explicit entity IDs (`RF-001`, `LAW-021`) |
| 🌐 | **REST API (17 endpoints)** | Auth, rate limiting, health probes, Prometheus metrics |
| 📦 | **Single Binary** | No runtime, cross-platform (macOS, Linux, Windows) |
| 🔌 | **Local Embeddings** | fastembed (ONNX Runtime), zero-config semantic search |
| 🐳 | **Docker Support** | Multi-stage build with health checks |

> ¹ Duplicate Code (SMELL-13) and Shotgun Surgery (SMELL-09) require multi-file context and are skipped in single-file mode.

---

## Installation

### Option 1: cargo-binstall (Recommended)

```bash
cargo binstall episteme    # downloads pre-built binary — no compilation
epis install cursor        # seeds data + starts API server + installs agents
```

If you don't have cargo-binstall yet: `cargo install cargo-binstall`

> After `epis install`, the HTTP API server starts automatically on port 58302. MCP is still available -- see `registry/mcp.json` for manual setup.

### Option 2: From Source

```bash
git clone https://github.com/epicsagas/Episteme.git
cd Episteme && cargo build --release
```

Then run the binary for your platform:

| Platform | Command |
|----------|---------|
| **macOS / Linux** | `./target/release/epis install --local cursor` |
| **Windows** | `.\target\release\episteme.exe install --local cursor` |

### Option 3: Docker (No Rust Required)

```bash
docker-compose up -d
```

Add to your MCP config file:

| Tool | Config file path |
|------|-----------------|
| Claude Code | `~/.claude.json` |
| Cursor | `.cursor/mcp.json` |
| VS Code (Copilot) | `.vscode/mcp.json` |

```json
{
  "mcpServers": {
    "episteme": {
      "command": "docker",
      "args": ["exec", "-i", "episteme-api", "episteme", "mcp"]
    }
  }
}
```

For HTTP transport with bearer token authentication:
```json
{
  "mcpServers": {
    "episteme": {
      "command": "docker",
      "args": ["exec", "-i", "episteme-api", "episteme", "mcp"],
      "headers": {
        "Authorization": "Bearer epis-a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6"
      }
    }
  }
}
```

### Option 4: Pre-built Binaries (No Rust Required)

Download the latest binary for your platform from [GitHub Releases](https://github.com/epicsagas/Episteme/releases):

| Platform | File |
|----------|------|
| **macOS** (Apple Silicon) | `episteme-aarch64-apple-darwin.tar.xz` |
| **Linux** (x86_64) | `episteme-x86_64-unknown-linux-gnu.tar.xz` |

```bash
# macOS / Linux
tar xzf episteme-*.tar.gz
sudo mv episteme /usr/local/bin/

# Windows — extract the zip and add episteme.exe to your PATH
```

Then install:
```bash
epis install cursor
```

### Verify

```bash
epis --version
epis stats
epis explore "strategy pattern"    # explore the knowledge graph
```

Or from inside Claude Code / Codex CLI:

```
/episteme verify
```

---

## HTTP API Endpoints

> Episteme runs as an always-on HTTP API server on port 58302. Skills and agents call `curl http://localhost:58302/...` instead of MCP tools. MCP is still available for manual setup -- see `registry/mcp.json`.

### API Endpoints

#### Knowledge Graph

| Method | Endpoint | Purpose |
|--------|----------|---------|
| **GET** | `/health` | Health check |
| **GET** | `/search?q=...` | Search knowledge graph |
| **GET** | `/graph/{id}` | Get entity by ID |
| **GET** | `/graph/{id}/neighbors` | Get related entities |
| **POST** | `/graph/path` | Find path between two entities |

#### Code Analysis

| Method | Endpoint | Purpose |
|--------|----------|---------|
| **POST** | `/analyze` | Detect code smells |
| **POST** | `/refactor` | Suggest refactorings |

#### Tacit Knowledge

| Method | Endpoint | Purpose |
|--------|----------|---------|
| **POST** | `/insights` | Add team insight |

### 9 MCP Tools (Legacy)

#### Canonical Knowledge (6 tools)

| Tool | Purpose | Example Use |
|------|---------|-------------|
| **`search_knowledge`** | Semantic search across all entities | "Find patterns for retry logic" |
| **`get_entity`** | Get details for specific entity by ID | "Explain Strategy Pattern (DP-023)" |
| **`get_neighbors`** | Explore related entities | "What refactorings solve Long Method?" |
| **`find_path`** | Find connection between two entities | "How does SRP relate to Extract Class?" |
| **`analyze_code`** | Detect code smells via regex/AST analysis | "Review this payment validation code" |
| **`suggest_refactorings`** | Ranked refactoring suggestions | "What should I refactor in this class?" |

#### Tacit Knowledge (3 tools)

| Tool | Purpose | Example Use |
|------|---------|-------------|
| **`add_insight`** | Record team decisions, lessons learned | "We chose event-driven over polling for X reason" |
| **`search_insights`** | Search past team knowledge | "What did we decide about auth middleware?" |
| **`confirm_links`** | Validate auto-detected links to canonical entities | Confirm TK-001 relates to SMELL-03 |

Episteme stores tacit knowledge in a separate database (`~/.episteme/user_knowledge.db`) and merges it with the canonical graph at runtime via a composite layer. Team insights get auto-linked to patterns, laws, and smells — turning experience into traversable knowledge.

See [Tacit Knowledge Architecture](docs/tacit-knowledge.md) for the full design.

### 4 Specialized Agents (Connected Network)

Agents work together — each analysis ends with **Next Steps** options that hand off to other agents.

| Agent | When to Use | Key Capability | Hands off to |
|-------|-------------|----------------|--------------|
| **`code-reviewer`** | Code smells, SOLID violations | Causation analysis (root cause → downstream symptoms) | advisor, architecture-analyst, refactoring-expert |
| **`episteme-advisor`** | Engineering decisions, trade-offs | Multi-entity trade-off chains with action plans | code-reviewer, architecture-analyst, researcher |
| **`episteme-researcher`** | Knowledge graph exploration | Connection maps between patterns, laws, smells | advisor, code-reviewer |
| **`architecture-analyst`** | Architecture evaluation against laws | Compliance scoring with risk-weighted assessment | advisor, code-reviewer, researcher |

**Workflow example**: `code-reviewer` detects God Object → traces causation to 3 downstream smells → offers "Apply RF-018" (→ refactoring-expert) or "Deep dive root cause" (→ episteme-advisor) or "Architecture check" (→ architecture-analyst).

[Full MCP Integration Guide](docs/mcp-integration-guide.md)

---

## CLI Usage

```bash
# Analyze code for smells
epis analyze my_code.py --language python --json
episteme infer my_code.py

# Explore the knowledge graph
epis explore "strategy pattern"
epis graph path DP-005 RF-001   # e.g. Factory Method → Extract Method

# Build the RAG index
epis build

# Start servers
epis api              # REST API on :58302
episteme mcp --http       # MCP server on :43175 (legacy)
episteme web --port 8080  # Web UI (interactive graph explorer)

# Distribution packaging
episteme dist --out-dir release/
```

---

## Documentation

| Document | Description |
|----------|-------------|
| [Quick Start](QUICKSTART.md) | Step-by-step setup, first run, troubleshooting |
| [MCP Integration Guide](docs/mcp-integration-guide.md) | Tool reference, agent examples, conversation flows |
| [Tacit Knowledge Architecture](docs/tacit-knowledge.md) | Two-database design, insight lifecycle, schema |
| [Alcove Ecosystem Comparison](docs/alcove-ecosystem.md) | Storage models, search capabilities, use-case matrix |
| [Alcove Integration Guide](docs/alcove-integration.md) | Dual-context workflows, setup, best practices |
| [API Reference](docs/api.md) | REST endpoints, authentication, examples |
| [Distribution](docs/distribution.md) | Release packaging and deployment |
| [Development & Contributing](DEVELOPMENT.md) | Architecture, how to contribute |
| [Changelog](CHANGELOG.md) | Release history and version notes |

---

## Configuration

### Environment Variables

```bash
# Data locations
EPISTEME_DATA_DIR=~/.episteme/data
EPISTEME_DB_PATH=~/.episteme/db/episteme.db

# API server
EPISTEME_API_HOST=0.0.0.0
EPISTEME_API_PORT=58302
EPISTEME_API_KEY=your-secret-key

# MCP server
EPISTEME_MCP_HOST=127.0.0.1
EPISTEME_MCP_PORT=43175
EPISTEME_MCP_TOKEN=epis-a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6
```

---

## Troubleshooting

**`episteme` command not found after install**

| Platform | Fix |
|----------|-----|
| **macOS / Linux** | `export PATH="$HOME/.cargo/bin:$PATH"` — add to `~/.bashrc` or `~/.zshrc` to persist |
| **Windows** | Add `%USERPROFILE%\.cargo\bin` to your system PATH, or open a new terminal |

**MCP tools not appearing in Claude Code / Cursor**

The HTTP API server starts automatically on port 58302 after `epis install`. Skills use `curl http://localhost:58302/...` to interact with Episteme. MCP is still available for manual setup -- see `registry/mcp.json`.

**Port already in use**
```bash
epis api --port 58303   # use a different port
```

**Slow first startup**

Episteme builds a local embedding index on first run. This takes 30–60 seconds and is a one-time cost. Subsequent starts are instant.

**Compilation errors during `cargo install`**

Ensure Rust 1.95+ is installed:
```bash
rustup update stable
rustup show   # confirm active toolchain
```

> More help: [QUICKSTART.md troubleshooting section](QUICKSTART.md#troubleshooting) · [Open an issue](https://github.com/epicsagas/Episteme/issues)

---

## Ecosystem: Alcove Integration

Episteme has two layers for capturing knowledge: the **canonical graph** (universal patterns, laws, smells) and the **tacit knowledge layer** (TK-* — team insights auto-linked to canonical entities). For richer project documentation — architecture decisions, coding conventions, onboarding guides, technical debt registers — **[Alcove](https://github.com/epicsagas/alcove)** is the recommended companion.

```mermaid
flowchart LR
    U["Developer"] --> A["AI Agent"]
    A -->|"What pattern applies?"| E["Episteme<br/>Canonical Graph"]
    A -->|"Quick insight?"| T["Episteme<br/>Tacit Knowledge (TK-*)"]
    A -->|"Team decisions?"| L["Alcove<br/>Project Docs"]
    E --> R["Grounded Recommendation"]
    T --> R
    L --> R
```

### Episteme vs Alcove — when to use which

| Scenario | Use | Why |
|----------|-----|-----|
| Detect code smells in a module | **Episteme** `analyze_code` | Regex/AST detection + ranked refactoring suggestions |
| Record a momentary insight ("we always hit N+1 here") | **Episteme** `add_insight` | Auto-links to relevant canonical entities (SMELL-*, LAW-*) |
| Find relationship between SRP and Extract Class | **Episteme** `find_path` | Multi-hop graph traversal across entity types |
| Start documentation for a new project | **Alcove** `init_project` | 7 core templates (PRD, ARCHITECTURE, DECISIONS, ...) auto-generated |
| Record a formal architecture decision (ADR) | **Alcove** DECISIONS.md | Structured ADR format with context, options, consequences |
| Check if docs are outdated or have broken links | **Alcove** `lint_project` | Detects WIP/TODO/DEPRECATED markers, orphan files, stale dates |
| Enforce naming conventions or required sections | **Alcove** `validate_docs` | Policy-based validation with pass/warn/fail |
| Import Obsidian notes for agent access | **Alcove** `promote_document` | Symlink vaults + BM25/vector indexing |
| Ground a recommendation in both principles and team rules | **Both** | Universal knowledge + team-specific constraints |

### Tacit Knowledge (TK-*) vs Alcove Docs

Episteme's tacit knowledge layer is designed for **short, momentary insights** that auto-connect to the knowledge graph — "we chose event-driven over polling because X", auto-linked to DP-018 (Observer) and LAW-012 (Fail Fast). Alcove handles **structured, long-lived documentation** — full ADRs with sections, architecture diagrams, coding standards, onboarding checklists.

| | Episteme TK-* | Alcove |
|---|---|---|
| **Granularity** | Atomic free-text insight | Structured multi-section document |
| **Auto-linking** | Keyword detection → canonical entities | wikilinks between docs |
| **Lifecycle** | Create + search | Full CRUD + validate + lint + audit + backup |
| **Search** | FTS5 keyword | BM25 + vector hybrid (CJK support) |
| **Best for** | Quick observations, lessons learned | Formal decisions, project scaffolding, doc governance |

Alcove manages 3 tiers of documentation (7 core + 19 supplementary + 15 public files), provides BM25 + vector hybrid search with CJK support, and integrates with Obsidian vaults. It includes policy validation, semantic linting (broken links, stale markers, orphans), and git-based backups.

**Full analysis**: [Alcove Ecosystem Comparison](docs/alcove-ecosystem.md) — storage models, search capabilities, feature completeness, and use-case decision matrix.

**Usage patterns**: [Alcove Integration Guide](docs/alcove-integration.md) — agent workflows, code review with dual context, and setup instructions.

---

## Roadmap

**Released**
- [x] `epis install` — one-command data setup from GitHub Releases
- [x] Homebrew tap (`epicsagas/tap/episteme`) — macOS Apple Silicon + Linux x86_64
- [x] Claude Code & Codex CLI plugin marketplace support
- [x] README translations — 9 languages (ko, ja, zh-CN, zh-TW, de, fr, es, pt, hi)

**Planned**
- [ ] **Cross-platform builds** — Migrate `fastembed` → `candle` (Pure Rust) to support Intel macOS, Windows, Linux ARM64 ([#32](https://github.com/epicsagas/Episteme/issues/32))
- [ ] **Custom Entities** — Add team-specific patterns and code smells
- [ ] **Multilingual Metadata** — Entity titles and summaries in CJK languages
- [ ] **Interactive Tutorials** — In-app guided tours for MCP tools
- [ ] **Team Metrics** — Aggregate pattern usage across organization

---

## Contributing

Contributions welcome! See [DEVELOPMENT.md](DEVELOPMENT.md) for the architecture overview and contribution guide.

```bash
# Run tests
cargo test

# Lint
cargo clippy -- -D warnings

# Format
cargo fmt
```

Questions? [Open a discussion](https://github.com/epicsagas/Episteme/discussions) or [file an issue](https://github.com/epicsagas/Episteme/issues).

---

## License

Apache 2.0 — see [LICENSE](LICENSE) for details.
