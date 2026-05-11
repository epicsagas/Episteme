# Episteme — Schnellstartanleitung

Starten Sie mit Episteme in weniger als 2 Minuten.

---

## Voraussetzungen

- **Rust 1.95+** (Edition 2024 erforderlich) — [Installation über rustup](https://rustup.rs)
- Internetverbindung (für den ersten Datenimport)

---

## Option 1: KI-Tool-Integration (Empfohlen)

**Ideal für:** Claude Code, Cursor, Codex, Gemini-Benutzer

```bash
# 1. Episteme installieren
cargo install --git https://github.com/epicsagas/Episteme

# 2. In Ihr KI-Tool installieren (lädt Daten herunter, konfiguriert MCP, kopiert Agenten)
epis install claude      # Claude Code
epis install cursor      # Cursor
epis install codex       # OpenAI Codex
epis install gemini      # Gemini CLI
epis install all         # Alle Tools gleichzeitig
```

> Wenn `epis install claude` beim Herunterladen der Daten fehlschlägt, verwenden Sie stattdessen die Quellcode-Installation unten.

**Das war's.** Starten Sie Ihr KI-Tool neu und Episteme ist aktiv.

---

## Option 2: Docker (Kein Rust erforderlich)

```bash
docker-compose up -d

# Zugriff
# API:       http://localhost:8000
# Health:    http://localhost:8000/health
```

Für die MCP-Integration über Docker fügen Sie Folgendes zu Ihrer MCP-Konfiguration hinzu:
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

## Option 3: Aus dem Quellcode

```bash
git clone https://github.com/epicsagas/Episteme.git
cd Episteme

# Kompilieren
cargo build --release

# Daten importieren und Vektor-DB erstellen (Build läuft automatisch)
./target/release/epis install --local
```

---

## Graph-Visualisierung

Episteme enthält einen interaktiven D3-force-Graph-Viewer:

```bash
episteme web               # Standard: http://localhost:8080
episteme web --port 9001   # Benutzerdefinierter Port
episteme web --host 0.0.0.0 --port 8080  # Im Netzwerk freigeben
```

---

## Häufige Befehle

```bash
# Code auf Smells analysieren
epis analyze my_code.py --language python
epis analyze my_code.py --json

# Refactoring-Vorschläge erhalten
episteme infer my_code.py --top-k 5

# Wissensgraph erkunden
epis explore "strategy pattern"
epis graph path DP-005 RF-001

# Server starten
epis api              # REST-API auf :8000
episteme mcp --http       # MCP-Server auf :43175
episteme web --port 8080  # Web-UI

# MCP-Hintergrunddienst (HTTP-Proxy)
epis service start
epis service status
epis service stop

# Release-Archiv erstellen
episteme dist --out-dir release
```

---

## Fehlerbehebung

### "Datenbank nicht gefunden"
```bash
epis install claude   # Datenarchiv erneut herunterladen
# oder
epis install --local
```

### "Port bereits belegt"
```bash
episteme web --port 9001
epis api --port 9000
```

---

## Nächste Schritte

- **[README](../../README.md)** — Vollständige Funktionsübersicht und Architektur
- **[MCP-Integrationsleitfaden](./mcp-integration-guide.md)** — Tool-Referenz und Agenten-Beispiele
- **[API-Referenz](./api.md)** — REST-Endpunkte
- **[Mitwirken](../../CONTRIBUTING.md)** — Entwicklungsworkflow
