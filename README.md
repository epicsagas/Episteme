<h1 align="center">Syntagma</h1>

<p align="center"><b>Knowledge Graph for Software Engineering</b></p>

<p align="center"><sub>Syntagma (συν ταγμα) — Greek for "organized system" or "discernment"</sub></p>

<p align="center">An offline-first, single-binary knowledge graph that connects design patterns, refactoring techniques, and software laws through semantic relationships.<br><b>Built for AI agents first</b> — integrate software engineering expertise directly into Claude Code, Cursor, and other MCP-compatible tools.</p>

<p align="center">Written in Rust · Single binary · Fully offline</p>

---

<p align="center">
    <a href="https://github.com/epicsagas/Syntagma/actions"><img src="https://img.shields.io/github/actions/workflow/status/epicsagas/Syntagma/ci.yml?branch=main&label=CI" alt="CI" /></a>
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.95+-orange.svg" alt="rust-lang" /></a>
    <a href="https://crates.io/crates/syntagma"><img src="https://img.shields.io/badge/crates.io-v0.1.0-orange.svg" alt="crates.io" /></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License" /></a>
    <a href="https://buymeacoffee.com/epicsaga"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-FFDD00?style=flat&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me a Coffee" /></a>
</p>

<p align="center">
  English |
  <a href="docs/i18n/README.ja.md">日本語</a> |
  <a href="docs/i18n/README.ko.md">한국어</a> |
  <a href="docs/i18n/README.de.md">Deutsch</a> |
  <a href="docs/i18n/README.fr.md">Français</a> |
  <a href="docs/i18n/README.zh-CN.md">简体中文</a> |
  <a href="docs/i18n/README.zh-TW.md">繁體中文</a> |
  <a href="docs/i18n/README.pt.md">Português</a> |
  <a href="docs/i18n/README.es.md">Español</a> |
  <a href="docs/i18n/README.hi.md">हिन्दी</a>
</p>

---

<img src="docs/assets/features.png" align="center" width="100%" alt="Syntagma Features Overview" />

---

## Quick Start

> **Prerequisites:** Rust 1.95+ via [rustup](https://rustup.rs) · **No Rust?** See [Docker](#option-3-docker-no-rust-required) or [pre-built binaries](#option-4-pre-built-binaries-no-rust-required).

**1. Install Rust (if not already installed)**

| OS | Command |
|----|---------|
| **macOS / Linux** | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **Windows** | Download and run [`rustup-init.exe`](https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe) |

After installing, open a **new terminal** (or run `source "$HOME/.cargo/env"` on macOS/Linux).

**2. Install Syntagma (first build takes 3–5 min)**

```bash
cargo install --git https://github.com/epicsagas/Syntagma
```

**3. Seed data + wire up your AI tool**

```bash
syntagma install claude    # or: cursor, codex, gemini
```

**4. Verify**

```bash
syntagma --version
syntagma stats
```

That's it. Restart Claude Code and Syntagma tools are ready.

### Try it in 30 seconds

**Option A — CLI:** Point it at any file in your project.

```bash
syntagma analyze src/domain/engine.rs
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

Syntagma auto-triggers — no special syntax needed. It maps your description to the knowledge graph and returns ranked, citable results.

---

## Why Syntagma?

LLMs already know what the Strategy pattern is. They can recite SOLID principles, list GoF patterns, and explain code smells. So why does this project exist?

**The gap isn't knowledge — it's structured, connected reasoning.**

When you ask an LLM "how do I fix a God Object?", it gives you a reasonable answer. But the answer changes between conversations, lacks traceability, and doesn't connect the problem to its root causes or downstream consequences. Syntagma turns isolated facts into a traversable graph where every recommendation is grounded, citable, and connected to the broader design landscape.

### How is this different from just prompting an LLM well?

| | Well-crafted LLM prompt | Syntagma + LLM |
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

The MCP integration auto-triggers on problem descriptions. When a user says "this class does too much", the agent doesn't need to know to ask about God Object — Syntagma maps the complaint to `SMELL-03`, surfaces ranked refactorings, and traces the violation back to first principles. This turns a vague complaint into a structured remediation plan.
</details>

<details>
<summary><b>2. When you want to reduce token consumption — not burn it on explanations</b></summary>

Without Syntagma, an LLM answers "how do I fix a God Object?" by explaining the smell, listing refactorings, describing SOLID principles, and walking through each option — hundreds of tokens per response. With Syntagma, one MCP tool call returns `SMELL-03 → RF-018 (0.89) → LAW-001`. Same expertise at a fraction of the token budget.
</details>

<details>
<summary><b>3. When you need code analysis connected to remediation — not just detection</b></summary>

Tools like SonarQube detect smells. LLMs can suggest patterns. Syntagma does both and connects them: detect Long Method → trace to the laws it violates → rank the refactorings that solve it → show what patterns enforce those refactorings.
</details>

<details>
<summary><b>4. When isolated pattern knowledge isn't enough — you need the relationships</b></summary>

Knowing what Extract Method does is table stakes. Knowing that it *solves* Long Method (SMELL-01), which *violates* Single Responsibility (LAW-001), which is *enforced by* Facade Pattern (DP-012) — that's a reasoning chain an LLM can't reliably construct on its own. Syntagma's 201 semantic relations let AI agents traverse these paths deterministically.
</details>

<details>
<summary><b>5. When you're making architecture decisions and need evidence, not opinions</b></summary>

"Should I use microservices?" — Syntagma connects the question to Conway's Law (LAW-017), SRP (LAW-001), and the Strangler Fig pattern (DP-026), then shows how they relate. Decisions become traceable to engineering laws, not blog posts.
</details>

<details>
<summary><b>6. When you need consistent, citable engineering advice — not hallucinated recommendations</b></summary>

Every finding references explicit entity IDs (`DP-005`, `RF-001`, `LAW-021`). Recommendations come with priority scores and effort estimates. The same query always returns the same structured answer.
</details>

<details>
<summary><b>7. When you're working in an air-gapped or restricted network</b></summary>

Syntagma runs entirely offline: single binary, local SQLite database, local embeddings via fastembed (ONNX Runtime). No telemetry, no phone-home, no external API calls. Your code and analysis results never leave your machine.
</details>

---

## Installation

### Option 1: One Command (Recommended)

```bash
# First build takes 3–5 minutes — this is normal
cargo install --git https://github.com/epicsagas/Syntagma
syntagma install claude    # seeds data + wires up MCP + installs agents
```

> After `syntagma install claude`, **restart Claude Code** for the MCP tools and agents to appear.

### Option 2: From Source

```bash
git clone https://github.com/epicsagas/Syntagma.git
cd Syntagma && cargo build --release
```

Then run the binary for your platform:

| Platform | Command |
|----------|---------|
| **macOS / Linux** | `./target/release/syntagma install --local claude` |
| **Windows** | `.\target\release\syntagma.exe install --local claude` |

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
    "syntagma": {
      "command": "docker",
      "args": ["exec", "-i", "syntagma-api", "syntagma", "mcp"]
    }
  }
}
```

### Option 4: Pre-built Binaries (No Rust Required)

Download the latest binary for your platform from [GitHub Releases](https://github.com/epicsagas/Syntagma/releases):

| Platform | File |
|----------|------|
| **macOS** (Apple Silicon) | `syntagma-aarch64-apple-darwin.tar.gz` |
| **macOS** (Intel) | `syntagma-x86_64-apple-darwin.tar.gz` |
| **Linux** (x86_64) | `syntagma-x86_64-unknown-linux-gnu.tar.gz` |
| **Linux** (ARM64) | `syntagma-aarch64-unknown-linux-gnu.tar.gz` |
| **Windows** (x86_64) | `syntagma-x86_64-pc-windows-msvc.zip` |

```bash
# macOS / Linux
tar xzf syntagma-*.tar.gz
sudo mv syntagma /usr/local/bin/

# Windows — extract the zip and add syntagma.exe to your PATH
```

Then install:
```bash
syntagma install claude    # or: cursor, codex, gemini
```

### Verify

```bash
syntagma --version
syntagma stats
syntagma explore "strategy pattern"    # explore the knowledge graph
```

---

## MCP Tools & Agents

> **What is MCP?** The [Model Context Protocol](https://modelcontextprotocol.io) is an open standard that lets AI tools call external services. Syntagma exposes its knowledge graph as MCP tools that Claude Code, Cursor, and other compatible editors can call automatically.

### 6 MCP Tools

| Tool | Purpose | Example Use |
|------|---------|-------------|
| **`search_knowledge`** | Semantic search across all entities | "Find patterns for retry logic" |
| **`get_entity`** | Get details for specific entity by ID | "Explain Strategy Pattern (DP-023)" |
| **`get_neighbors`** | Explore related entities | "What refactorings solve Long Method?" |
| **`find_path`** | Find connection between two entities | "How does SRP relate to Extract Class?" |
| **`analyze_code`** | Detect code smells via regex/AST analysis | "Review this payment validation code" |
| **`suggest_refactorings`** | Ranked refactoring suggestions | "What should I refactor in this class?" |

### 4 Specialized Agents (Connected Network)

Agents work together — each analysis ends with **Next Steps** options that hand off to other agents.

| Agent | When to Use | Key Capability | Hands off to |
|-------|-------------|----------------|--------------|
| **`code-reviewer`** | Code smells, SOLID violations | Causation analysis (root cause → downstream symptoms) | advisor, architecture-analyst, refactoring-expert |
| **`syntagma-advisor`** | Engineering decisions, trade-offs | Multi-entity trade-off chains with action plans | code-reviewer, architecture-analyst, researcher |
| **`syntagma-researcher`** | Knowledge graph exploration | Connection maps between patterns, laws, smells | advisor, code-reviewer |
| **`architecture-analyst`** | Architecture evaluation against laws | Compliance scoring with risk-weighted assessment | advisor, code-reviewer, researcher |

**Workflow example**: `code-reviewer` detects God Object → traces causation to 3 downstream smells → offers "Apply RF-018" (→ refactoring-expert) or "Deep dive root cause" (→ syntagma-advisor) or "Architecture check" (→ architecture-analyst).

[Full MCP Integration Guide](docs/mcp-integration-guide.md)

---

## CLI Usage

```bash
# Analyze code for smells
syntagma analyze my_code.py --language python --json
syntagma infer my_code.py

# Explore the knowledge graph
syntagma explore "strategy pattern"
syntagma graph path DP-005 RF-001   # e.g. Factory Method → Extract Method

# Build the RAG index
syntagma build

# Start servers
syntagma api              # REST API on :8000
syntagma mcp --http       # MCP server on :43175
syntagma web --port 8080  # Web UI (interactive graph explorer)

# Distribution packaging
syntagma dist --out-dir release/
```

---

## Features

### Knowledge Base
- **22 GoF Design Patterns** — Complete catalog with real-world examples
- **66 Refactoring Techniques** — From Fowler's catalog with code samples
- **56 Software Laws & Principles** — SOLID, Conway's Law, CAP Theorem, etc.
- **17 Code Smell Types** — Long Method, God Object, Feature Envy, etc. ¹
- **201 Semantic Relations** — "solves", "enforces", "violates", "relates_to"

### AI-First Design
- **MCP Integration** — 6 specialized tools for high-fidelity AI agent interaction
- **4 Connected Agents** — Causation analysis, interactive follow-ups, and cross-agent handoffs
- **10 Language Support** — Python (AST), Java, TypeScript, Go, Rust, C++, C#, PHP, Ruby, Kotlin
- **Deterministic Analysis** — AST-based Python detection + regex-based multi-language support
- **Citable Knowledge** — Every finding links to explicit entity IDs (e.g., `RF-001`, `LAW-021`)
- **Workflow Chains** — Multi-step pipelines: Code Review → Causation Analysis → Refactoring → Verification

### Production Ready
- **REST API** — 17 endpoints with authentication and rate limiting
- **Single Binary** — No runtime dependencies, cross-platform
- **Local Embeddings** — fastembed (ONNX Runtime) for zero-config semantic search
- **Interactive Visualization** — Web-based graph explorer (`syntagma web`)
- **Docker Support** — Multi-stage build with health checks
- **Monitoring** — Prometheus metrics endpoint

> ¹ Duplicate Code (SMELL-13) and Shotgun Surgery (SMELL-09) require multi-file context and are skipped in single-file mode.

---

## Documentation

| Document | Description |
|----------|-------------|
| [Quick Start](QUICKSTART.md) | Step-by-step setup, first run, troubleshooting |
| [MCP Integration Guide](docs/mcp-integration-guide.md) | Tool reference, agent examples, conversation flows |
| [API Reference](docs/api.md) | REST endpoints, authentication, examples |
| [Distribution](docs/distribution.md) | Release packaging and deployment |
| [Development & Contributing](DEVELOPMENT.md) | Architecture, how to contribute |
| [Changelog](CHANGELOG.md) | Release history and version notes |

---

## Configuration

### Environment Variables

```bash
# Data locations
SYNTAGMA_DATA_DIR=~/.syntagma/data
SYNTAGMA_DB_PATH=~/.syntagma/db/syntagma.db

# API server
SYNTAGMA_API_HOST=0.0.0.0
SYNTAGMA_API_PORT=8000
SYNTAGMA_API_KEY=your-secret-key

# MCP server
SYNTAGMA_MCP_HOST=127.0.0.1
SYNTAGMA_MCP_PORT=43175
```

---

## Troubleshooting

**`syntagma` command not found after install**

| Platform | Fix |
|----------|-----|
| **macOS / Linux** | `export PATH="$HOME/.cargo/bin:$PATH"` — add to `~/.bashrc` or `~/.zshrc` to persist |
| **Windows** | Add `%USERPROFILE%\.cargo\bin` to your system PATH, or open a new terminal |

**MCP tools not appearing in Claude Code / Cursor**

Restart the editor after running `syntagma install`. If still missing, check the config was written:
```bash
cat ~/.claude.json   # Claude Code
```

**Port already in use**
```bash
syntagma mcp --http --port 43176   # use a different port
```

**Slow first startup**

Syntagma builds a local embedding index on first run. This takes 30–60 seconds and is a one-time cost. Subsequent starts are instant.

**Compilation errors during `cargo install`**

Ensure Rust 1.95+ is installed:
```bash
rustup update stable
rustup show   # confirm active toolchain
```

> More help: [QUICKSTART.md troubleshooting section](QUICKSTART.md#troubleshooting) · [Open an issue](https://github.com/epicsagas/Syntagma/issues)

---

## Roadmap

- [ ] **Interactive Tutorials** — In-app guided tours for MCP tools
- [ ] **Team Metrics** — Aggregate pattern usage across organization
- [ ] **Custom Entities** — Add team-specific patterns/smells
- [ ] **IDE Plugins** — VSCode, IntelliJ native integrations
- [ ] **Multilingual Docs** — Knowledge base in Korean, Japanese, Chinese

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

Questions? [Open a discussion](https://github.com/epicsagas/Syntagma/discussions) or [file an issue](https://github.com/epicsagas/Syntagma/issues).

---

## License

Apache 2.0 — see [LICENSE](LICENSE) for details.
