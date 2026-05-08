# Episteme — Quick Start Guide

Get up and running with Episteme in under 2 minutes.

---

## Prerequisites

- **Rust 1.95+** (edition 2024 required) — [Install via rustup](https://rustup.rs)
- Internet connection (for initial data download)

---

## Option 1: AI Tool Integration (Recommended)

**Perfect for:** Claude Code, Cursor, Codex, Gemini users

```bash
# 1. Install Episteme
cargo install --git https://github.com/epicsagas/Episteme

# 2. Install into your AI tool (downloads data, configures MCP, copies agents)
epis install claude      # Claude Code
epis install cursor      # Cursor
epis install codex       # OpenAI Codex
epis install gemini      # Gemini CLI
epis install all         # All tools at once
```

> If `epis install claude` fails to download data, use the source install below instead.

**That's it.** Restart your AI tool and Episteme is active.

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
    "episteme": {
      "command": "docker",
      "args": ["exec", "-i", "episteme-api", "episteme", "mcp"]
    }
  }
}
```

---

## Option 3: From Source

```bash
git clone https://github.com/epicsagas/Episteme.git
cd Episteme

# Build
cargo build --release

# Seed data and build vector DB (build runs automatically)
./target/release/epis install --local
```

---

## Graph Visualization

Episteme includes an interactive D3-force graph viewer:

```bash
episteme web               # default: http://localhost:8080
episteme web --port 9001   # custom port
episteme web --host 0.0.0.0 --port 8080  # expose to network
```

---

## Common Commands

```bash
# Analyze code for smells
epis analyze my_code.py --language python
epis analyze my_code.py --json

# Get refactoring suggestions
episteme infer my_code.py --top-k 5

# Explore the knowledge graph
epis explore "strategy pattern"
epis graph path DP-005 RF-001

# Start servers
epis api              # REST API on :8000
episteme mcp --http       # MCP server on :43175
episteme web --port 8080  # Web UI

# Background MCP daemon (HTTP proxy)
epis service start
epis service status
epis service stop

# Create release archive
episteme dist --out-dir release
```

---

## Troubleshooting

### "Database not found"
```bash
epis install claude   # re-download data archive
# or
epis install --local
```

### "Port already in use"
```bash
episteme web --port 9001
epis api --port 9000
```

---

## Next Steps

- **[README](README.md)** — Full feature overview and architecture
- **[MCP Integration Guide](docs/mcp-integration-guide.md)** — Tool reference and agent examples
- **[API Reference](docs/api.md)** — REST endpoints
- **[Contributing](CONTRIBUTING.md)** — Development workflow
