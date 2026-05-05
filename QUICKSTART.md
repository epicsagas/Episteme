# Syntagma — Quick Start Guide

Get up and running with Syntagma in under 2 minutes.

---

## Prerequisites

- **Rust 1.85+** (edition 2024 required) — [Install via rustup](https://rustup.rs)
- Internet connection (for initial data download)

---

## Option 1: AI Tool Integration (Recommended)

**Perfect for:** Claude Code, Cursor, Codex, Gemini users

```bash
# 1. Install Syntagma
cargo install --git https://github.com/epicsagas/Syntagma

# 2. Install into your AI tool (downloads data, configures MCP, copies agents)
syntagma install claude      # Claude Code
syntagma install cursor      # Cursor
syntagma install codex       # OpenAI Codex
syntagma install gemini      # Gemini CLI
syntagma install all         # All tools at once
```

> If `syntagma install claude` fails to download data, use the source install below instead.

**That's it.** Restart your AI tool and Syntagma is active.

---

## Option 2: Docker (No Rust Required)

```bash
docker-compose up -d

# Access
# API:       http://localhost:8000
# Health:    http://localhost:8000/health
```

For MCP integration via Docker, add to your MCP config:
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

---

## Option 3: From Source

```bash
git clone https://github.com/epicsagas/Syntagma.git
cd Syntagma

# Build
cargo build --release

# Seed data and build vector DB (build runs automatically)
./target/release/syntagma install --local
```

---

## Graph Visualization

Syntagma includes an interactive D3-force graph viewer:

```bash
syntagma web               # default: http://localhost:8080
syntagma web --port 9001   # custom port
syntagma web --host 0.0.0.0 --port 8080  # expose to network
```

---

## Common Commands

```bash
# Analyze code for smells
syntagma analyze my_code.py --language python
syntagma analyze my_code.py --json

# Get refactoring suggestions
syntagma infer my_code.py --top-k 5

# Explore the knowledge graph
syntagma explore "strategy pattern"
syntagma graph path DP-005 RF-001

# Start servers
syntagma api              # REST API on :8000
syntagma mcp --http       # MCP server on :43175
syntagma web --port 8080  # Web UI

# Background MCP daemon (HTTP proxy)
syntagma service start
syntagma service status
syntagma service stop

# Create release archive
syntagma dist --out-dir release
```

---

## Troubleshooting

### "Database not found"
```bash
syntagma install claude   # re-download data archive
# or
syntagma install --local
```

### "Port already in use"
```bash
syntagma web --port 9001
syntagma api --port 9000
```

---

## Next Steps

- **[README](README.md)** — Full feature overview and architecture
- **[MCP Integration Guide](docs/mcp-integration-guide.md)** — Tool reference and agent examples
- **[API Reference](docs/api.md)** — REST endpoints
- **[Contributing](CONTRIBUTING.md)** — Development workflow
