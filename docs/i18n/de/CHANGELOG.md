# Changelog

Alle nennenswerten Änderungen an Episteme werden in dieser Datei dokumentiert.

Das Format basiert auf [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
und dieses Projekt befolgt [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- CLI: `explore` wurde in `search` umbenannt (alter Name funktioniert als veralteter Alias)
- CLI: `mcp` und `api` verwalten nun ihren vollständigen Dienstlebenszyklus (`start`, `stop`, `restart`, `status`, `enable [--now]`, `disable [--now]`)
- CLI: `service`-Befehl auf oberster Ebene ist veraltet — verwenden Sie stattdessen `mcp start/stop/restart/status/enable/disable`
- CLI: `mcp --http` ist veraltet — verwenden Sie `mcp start` für den HTTP-Daemon-Modus
- CLI: `launchd-install/uninstall/status` ist veraltet — verwenden Sie stattdessen `mcp enable/disable/status`
- `enable/disable` ist nun plattformübergreifend: macOS (launchd) und Linux (systemd User Unit)

### Added

- `api start/stop/restart/status/enable/disable` — REST-API-Daemon-Lebenszyklusverwaltung
- Linux systemd User Unit-Generierung für `mcp enable`

- **MCP-HTTP-Transport für Claude Code** — Transport-Auswahl-TUI, HTTP als Standard, launchd-Auto-Aktivierung
- **Agenten-Prompt-Auto-Installation** — `epis install` kopiert Episteme-Agenten-Prompts in `~/.claude/agents/`
- **Entitätsbeschreibungen** — Beschreibungsfeld automatisch aus Markdown-Quelldateien extrahiert, im Web-Viewer-Detailbereich angezeigt
- **Benchmark-Visualisierungs-SPA** — Trendanalyse, Query-Aufschlüsselungs-Dashboard
- **Web-Viewer-Redesign** — Sankey-Diagramm-Layout, Seitenleisten-Baum, Detailbereich, Verbesserungen der Subgraph-Lesbarkeit
- **MCP-Konfiguration Upsert** — Erneutes Ausführen von `epis install` aktualisiert den Transport bei abweichender Konfiguration (stdio ↔ HTTP)
- **MCP-YAML-Konfiguration** — `mcp.host` / `mcp.port` in `config.yaml` (yaml → env Fallback)
- **Monitoring** — Native und entfernte Prometheus-Scrape-Target-Unterstützung über Umgebungsvariablen
- **CI-Härtung** — cargo audit, gitleaks, SBOM-Generierung, gepinnte Action-SHAs
- **Release-Pipeline** — Windows-Target, crates.io-Veröffentlichung, Homebrew-Tap
- **God-Module-Architekturdiagnose-Beispiel** in `examples/`

### Changed

- **Installations-Assistent** — Alle Schritte (Transport, Redis, Telemetrie) in Vollbild-TUI migriert
- **Installationsablauf** — Erstellt RAG-Index automatisch nach dem Seeding, überspringt wenn DB bereits existiert
- **Wissensgraph** — Mit cross-entitären semantischen Relationen angereichert
- **Lizenz** — MIT → Apache-2.0

### Fixed

- Tokio-Laufzeit-Panic in synchronem `main()` für Telemetrie
- Suchqualität — NDCG-Messfehler behoben, hit@1-Genauigkeit auf 100% verbessert
- Such-Recall — Cross-Type-Boosting, Sparse-Entity-Behandlung, Intent-Synonyme
- fastembed-Modell-Cache auf `~/.episteme/models` gepinnt
- launchd-Bootstrap-UID-Substitution und Port-in-use-Behandlung
- CORS-Origins sind nun über `EPISTEME_CORS_ORIGINS` konfigurierbar

## [0.1.0] - 2026-05-03

### Added

- **Vollständiges Rust-Rewrite** — Kompletter Ersatz der Python-Codebasis durch idiomatisches Rust
- **Hexagonale Architektur** — `ports/` (Traits), `domain/` (Geschäftslogik), `adapters/` (Infrastruktur), `server/` (HTTP)
- **GenericParser-Framework** — 8 klammerbasierte Parser in `GenericParser` mit `ParserConfig` zusammengefasst; Regex-Muster über `OnceLock` mit `Box::leak` gecacht
- **Python-AST-Parsing** — `rustpython-parser` für präzise Python-Smell-Erkennung (Long Method, Large Class, God Object)
- **TieredAccum + build_detection()** — 14 identische Smell-Erkennungskonstruktionen in `detectors.rs` dedupliziert (1.253 → 591 Zeilen)
- **MCP-Modul-Zerlegung** — `EpistemeMCP` (675 Zeilen) in `mcp_search`, `mcp_graph`, `mcp_analysis`-Dienste aufgeteilt
- **CLI-Befehl-Zerlegung** — `main.rs` (1.741 Zeilen) in `commands/`-Modul mit `cli.rs` für clap-Definitionen aufgeteilt
- **API-Handler-Deduplizierung** — Doppelte `search`/`search_post` in gemeinsame `do_search()` zusammengeführt
- **16 Smell-Detektorfunktionen** — erhöht von 14, abdeckend alle GoF-Smell-Kategorien
- **17 REST-API-Endpunkte** — Health-Probes, Prometheus-Metriken, CORS, Rate-Limiting
- **Rate-Limiter-TTL-Eviction** — MAX_BUCKETS=10.000 mit 1-Stunden-TTL zur Verhinderung unbegrenzten Speicherwachstums
- **ReDoS-Mitigation** — Begrenzter ternärer Operator-Regex von `[^:]+` zu `[^:\n]{1,50}`
- **Lokale Embeddings** — fastembed (ONNX Runtime) für zero-config semantische Suche
- **Interaktiver Installations-Assistent** — TUI mit crossterm, Vim-Tastenbelegungen, Alternate Screen
- **Distributions-Verpackung** — `episteme dist`-Befehl zur Release-Archiverstellung mit automatischem DB-Bootstrap
- **Plattformübergreifende CI** — GitHub Actions Release-Workflow für linux/macOS (x86_64 + aarch64)
- **Multi-Stage-Dockerfile** — Rust-Builder + schlankes Debian-Runtime-Image

### Changed

- **Sprache**: Python 3.11+ → Rust (Edition 2024)
- **Web-Framework**: FastAPI → axum
- **Datenbank**: Python sqlite3 → rusqlite (gebündelt)
- **Embeddings**: sentence-transformers/PyTorch → fastembed/ONNX Runtime
- **CLI**: argparse → clap (derive)
- **Alle Regex-Muster gecacht** — Keine Rekompilierung auf Hot Paths über globalen `REGEX_CACHE`

### Removed

- Python-Laufzeitabhängigkeit
- ChromaDB-Abhängigkeit
- tree-sitter-Abhängigkeit
- PyPI-Veröffentlichungs-Workflow
- `episteme-hook` Standalone-Binary (war Python-only PyPI-Einstiegspunkt) — verwenden Sie stattdessen `episteme hooks ground|sniff|audit`

## [0.0.5] - 2026-04-30

### Added

- Graph-Visualisierungs-Web-UI (`episteme web`) mit D3-force
- Vorgefertigte Vektor-DB im Release-Archiv
- `epis install --local`-Flag für Entwicklungsworkflows
- 650+ semantische Relationen, die alle 161 Entitäten abdecken
- CI erzeugt Vektor-DB automatisch während des Releases

## [0.0.4] - 2026-04-29

### Added

- MCP-Server mit 6 Tools
- 4 spezialisierte Agenten
- `epis install`-Befehl
- `epis service`-Daemon-Verwaltung
- Hybridsuche (FTS5 + Vektor)
- Redis-Caching, GPU-Beschleunigung
- 10-Sprachen-Code-Smell-Erkennung
- Prometheus + Grafana-Monitoring
