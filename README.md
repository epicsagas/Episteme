# Syntagma: Knowledge Graph for Software Engineering

> Syntagma (συν ταγμα) - Greek for "organized system" or "discernment"

A production-ready knowledge graph system for software engineering that connects design patterns, refactoring techniques, and software laws through semantic relationships. **Built for AI agents first** — integrate software engineering expertise directly into Claude Code, Cursor, and other MCP-compatible tools.

Written in **Rust** for performance, safety, and single-binary deployment.

[![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)](https://github.com/epicsagas/Syntagma)
[![Rust](https://img.shields.io/badge/rust-1.82+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)

---

## Quick Start for AI Agents

### Option 1: MCP Integration (Recommended)

**For Claude Code / Cursor users** — Get software engineering knowledge in 2 commands:

```bash
# 1. Install Syntagma
cargo install --git https://github.com/epicsagas/Syntagma

# 2. Install into your AI tool
syntagma install claude    # or: cursor, codex, gemini
```

**That's it!** Restart Claude Code and you now have:

- 6 MCP tools (search, analyze code, suggest refactorings, etc.)
- 4 specialized agents (advisor, researcher, code-reviewer, architecture-analyst)
- Access to 22 patterns, 66 refactorings, 56 laws, 17 code smells
- Pre-built vector database — no `syntagma build` required

**Try it:**
```
User: "What's the best way to fix a God Object smell?"

Claude (using Syntagma MCP tools):
"The God Object anti-pattern (SMELL-03) violates Single Responsibility
Principle (LAW-001). Best refactorings:

1. Extract Class (RF-018) - Move related methods to new class
   Priority: 0.89 | Effort: Medium

2. Move Method (RF-023) - Relocate methods to appropriate classes
   Priority: 0.76 | Effort: Low

Start with Move Method for quick wins, then Extract Class for design."
```

[Full MCP Integration Guide](docs/mcp-integration-guide.md)

---

### Option 2: REST API

**For custom integrations** — Access via HTTP:

```bash
# Start API server
syntagma api

# Search knowledge
curl http://localhost:8000/search?q=strategy+pattern&top_k=3

# Analyze code
curl -X POST http://localhost:8000/analyze \
  -F "file=@my_code.py" \
  -F "language=python"

# Get refactoring suggestions
curl http://localhost:8000/refactor/SMELL-01
```

[API Documentation](docs/api.md)

---

### Option 3: Docker

```bash
docker-compose up -d

# API: http://localhost:8000
# Web: syntagma web --port 8080
```

---

## CLI Usage

```bash
# Analyze code for smells
syntagma analyze my_code.py --language python --json
syntagma infer my_code.py

# Explore the knowledge graph
syntagma explore "strategy pattern"
syntagma graph path DP-005 RF-001

# Build the RAG index
syntagma build

# Start servers
syntagma api              # REST API on :8000
syntagma mcp --http       # MCP server on :43175
syntagma web --port 8080  # Web UI

# Distribution packaging
syntagma dist --out-dir release/
```

---

## MCP Tools & Agents

### 6 MCP Tools

| Tool | Purpose | Example Use |
|------|---------|-------------|
| **`search_knowledge`** | Semantic search across all entities | "Find patterns for retry logic" |
| **`get_entity`** | Get details for specific entity by ID | "Explain Strategy Pattern (DP-023)" |
| **`get_neighbors`** | Explore related entities | "What refactorings solve Long Method?" |
| **`find_path`** | Find connection between two entities | "How does SRP relate to Extract Class?" |
| **`analyze_code`** | Detect code smells via regex/AST analysis | "Review this payment validation code" |
| **`suggest_refactorings`** | Ranked refactoring suggestions | "What should I refactor in this class?" |

### 4 Specialized Agents

| Agent | When to Use | Example Prompt |
|-------|-------------|----------------|
| **`syntagma-advisor`** | Engineering decisions, pattern selection | "Should I use Singleton for ConfigManager?" |
| **`syntagma-researcher`** | Explore knowledge graph relationships | "What laws does Observer Pattern enforce?" |
| **`code-reviewer`** | Review code for smells and SOLID violations | "Review this authentication module" |
| **`architecture-analyst`** | Evaluate architecture against laws/patterns | "Is this microservices design following Conway's Law?" |

[Full MCP Integration Guide](docs/mcp-integration-guide.md)

---

## Features

### Knowledge Base
- **22 GoF Design Patterns** - Complete catalog with real-world examples
- **66 Refactoring Techniques** - From Fowler's catalog with code samples
- **56 Software Laws & Principles** - SOLID, Conway's Law, CAP Theorem, etc.
- **17 Code Smell Types** - Long Method, God Object, Feature Envy, etc.
- **201 Semantic Relations** - "solves", "enforces", "violates", "relates_to"

### AI-First Design
- **MCP Integration** - 6 specialized tools for high-fidelity AI agent interaction
- **10 Language Support** - Python (AST), Java, TypeScript, Go, Rust, C++, C#, PHP, Ruby, Kotlin
- **Deterministic Analysis** - AST-based Python detection + regex-based multi-language support
- **Citable Knowledge** - Every finding links to explicit entity IDs (e.g., `RF-001`, `LAW-021`)

### Production Ready
- **REST API** - 17 endpoints with authentication and rate limiting
- **Single Binary** - No runtime dependencies, cross-platform
- **Local Embeddings** - fastembed (ONNX Runtime) for zero-config semantic search
- **Interactive Visualization** - Web-based graph explorer (`syntagma web`)
- **Docker Support** - Multi-stage build with health checks
- **Monitoring** - Prometheus metrics endpoint

---

## Architecture

```
syntagma (CLI binary)
├── src/
│   ├── commands/          # CLI subcommand handlers
│   │   ├── analysis.rs    # analyze, infer
│   │   ├── build.rs       # build (RAG pipeline)
│   │   ├── explore.rs     # explore (search/REPL)
│   │   ├── graph.rs       # graph queries
│   │   ├── install.rs     # install wizard
│   │   ├── service.rs     # MCP service management
│   │   └── other.rs       # api, mcp, web, telemetry, hooks
│   ├── adapters/          # Infrastructure layer
│   │   ├── regex_parsers.rs   # GenericParser (10 languages)
│   │   ├── search_engines.rs  # FTS5 + cosine similarity
│   │   ├── service.rs         # MCP HTTP daemon
│   │   └── ...
│   ├── domain/            # Business logic
│   │   ├── graph.rs       # KnowledgeGraph (BFS, subgraph, contradictions)
│   │   ├── detectors.rs   # 16 smell detectors
│   │   ├── engine.rs      # RefactoringInferenceEngine
│   │   └── summarizer.rs  # Detail-level response optimization
│   ├── server/            # HTTP layer (axum)
│   │   ├── api_routes.rs  # 17 REST endpoints
│   │   ├── mcp_handler.rs # MCP facade
│   │   ├── mcp_search.rs  # Search service
│   │   ├── mcp_graph.rs   # Graph service
│   │   └── mcp_analysis.rs # Code analysis service
│   └── ports/             # Traits (hexagonal boundaries)
```

**Tech Stack:** Rust, axum, rusqlite (SQLite + FTS5), fastembed (ONNX), clap, regex

---

## Installation

### Option 1: From Source (Recommended)

```bash
git clone https://github.com/epicsagas/Syntagma.git
cd Syntagma

cargo build --release

# Binary at target/release/syntagma
./target/release/syntagma install --local   # seeds data and builds DB automatically
```

### Option 2: Docker

```bash
docker-compose up -d
```

### Verify Installation

```bash
syntagma --version              # 0.1.0
syntagma stats                  # Knowledge graph statistics
syntagma analyze test.py        # Code smell detection
syntagma mcp                    # Start MCP server (stdio)
```

---

## Documentation

| Document | Description |
|----------|-------------|
| [MCP Integration Guide](docs/mcp-integration-guide.md) | Tool reference, agent examples, conversation flows |
| [API Reference](docs/api.md) | REST endpoints, authentication, examples |
| [Distribution](docs/distribution.md) | Release packaging and deployment |
| [Development](DEVELOPMENT.md) | Architecture, contributing guide |

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

## Roadmap

- [ ] **Interactive Tutorials** - In-app guided tours for MCP tools
- [ ] **Team Metrics** - Aggregate pattern usage across organization
- [ ] **Custom Entities** - Add team-specific patterns/smells
- [ ] **IDE Plugins** - VSCode, IntelliJ native integrations
- [ ] **Multilingual Docs** - Knowledge base in Korean, Japanese, Chinese

---

## Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md).

---

## License

APACHE-2.0 License - See [LICENSE](LICENSE) for details.
