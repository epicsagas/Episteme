# Episteme Entwicklungsleitfaden

**Projekt:** Episteme v0.1.0
**Sprache:** Rust (Edition 2024)
**Letzte Aktualisierung:** 2026-05-03

---

## Aktueller Status

| Komponente | Status | Details |
|------------|--------|---------|
| **Wissensbasis** | Fertiggestellt | 22 Muster, 66 Refactorings, 56 Gesetze, 23 Smells, 201 Relationen |
| **Code-Smell-Erkennung** | Produktiv | 16 Detektorfunktionen, 10 Sprachen |
| **REST-API** | Produktiv | 17 Endpunkte (axum), Rate-Limiting, Auth |
| **MCP-Server** | Produktiv | 6 Tools, stdio + HTTP-Transport |
| **RAG-Pipeline** | Produktiv | SQLite + FTS5 + fastembed (ONNX) |
| **Graph-Visualisierung** | Produktiv | Interaktive Web-UI mit D3-force |

---

## Architektur

Hexagonale Architektur (Ports & Adapters):

```
src/
├── commands/          # CLI-Unterbefehl-Handler (clap)
│   ├── analysis.rs    # analyze, infer
│   ├── build.rs       # build (RAG-Pipeline)
│   ├── explore.rs     # explore (Suche/REPL)
│   ├── graph.rs       # Graph-Abfragen
│   ├── install.rs     # Installations-Assistent (TUI)
│   ├── service.rs     # MCP-HTTP-Daemon-Verwaltung
│   └── other.rs       # api, mcp, web, telemetry, hooks
├── adapters/          # Infrastrukturschicht
│   ├── regex_parsers.rs   # GenericParser (10 Sprachen, OnceLock-Regex-Cache)
│   ├── python_ast_parser.rs  # Python-AST (rustpython-parser)
│   ├── search_engines.rs  # FTS5-Schlüsselwort + Kosinusähnlichkeit
│   ├── service.rs         # MCP-HTTP-Daemon
│   ├── sqlite_db.rs       # SQLite-Verbindungspool
│   ├── cache.rs           # Redis-Caching (optional)
│   └── ...
├── domain/            # Geschäftslogik (keine externen Abhängigkeiten)
│   ├── graph.rs       # KnowledgeGraph (BFS, Subgraph, Widersprüche, Jaccard)
│   ├── detectors.rs   # 16 Smell-Detektoren mit TieredAccum
│   ├── engine.rs      # RefactoringInferenceEngine + RefactoringRanker
│   ├── summarizer.rs  # Detail-Level-Antwortoptimierung
│   └── types.rs       # EntityType, RelationType, Kerntypen
├── server/            # HTTP-Schicht (axum)
│   ├── api_routes.rs  # 17 REST-Endpunkte
│   ├── mcp_handler.rs # MCP-Thin-Facade
│   ├── mcp_search.rs  # Suchdienst
│   ├── mcp_graph.rs   # Graph-Dienst
│   └── mcp_analysis.rs # Code-Analyse-Dienst
└── ports/             # Traits (hexagonale Grenzen)
    ├── parser.rs      # CodeParser-Trait
    ├── search.rs      # SearchEngine-Trait
    ├── graph.rs       # GraphStore-Trait
    └── embeddings.rs  # EmbeddingProvider-Trait
```

---

## Technologie-Stack

| Komponente | Technologie | Zweck |
|------------|-------------|-------|
| **Sprache** | Rust (Edition 2024) | Sicherheit, Leistung, Einzelnes Binary |
| **Web-Framework** | axum | REST-API + MCP-HTTP-Transport |
| **Datenbank** | rusqlite (gebündeltes SQLite) | Wissensgraph + Vektorspeicher |
| **Suche** | FTS5 + Kosinusähnlichkeit | Schlüsselwort + semantische Hybridsuche |
| **Embeddings** | fastembed (ONNX Runtime) | Lokale, konfigurationsfreie Embedding-Generierung |
| **CLI** | clap (derive) | 15 Unterbefehle |
| **Python-AST** | rustpython-parser | AST-basierte Python-Smell-Erkennung |
| **Andere Sprachen** | regex (OnceLock-gecacht) | GenericParser-Framework |

---

## Code-Smell-Detektoren (16)

| ID | Smell | Erkennung |
|----|-------|-----------|
| SMELL-01 | Long Method | LOC-Schwellenwert |
| SMELL-02 | Long Parameter List | Parameteranzahl |
| SMELL-03 | Primitive Obsession | Primitive-Parameter-Verhältnis |
| SMELL-04 | Large Class | Methoden- + Feldanzahl |
| SMELL-05 | Data Clumps | Wiederholte Parametergruppen (Stub) |
| SMELL-06 | Switch Statements | Switch/match-Anzahl |
| SMELL-07 | Data Class | Methoden-zu-Felder-Verhältnis |
| SMELL-08 | Temporary Field | Bedingte Feldnutzung (Stub) |
| SMELL-09 | Shotgun Surgery | Änderungskopplung (Stub) |
| SMELL-10 | Divergent Change | Methodenkohäsionsmetriken |
| SMELL-11 | Lazy Class | Niedrige LOC + Methodenanzahl |
| SMELL-12 | Speculative Generality | Abstrakt ohne konkret |
| SMELL-13 | Duplicate Code | Hash-basierte Ähnlichkeit (teilweise) |
| SMELL-14 | Middle Man | Delegationsverhältnis |
| SMELL-15 | Parallel Inheritance Hierarchies | Hierarchie-Spiegelung (Stub) |
| SMELL-16 | Comments | Kommentar-zu-Code-Verhältnis (Stub) |
| SMELL-17 | Dead Code | Unerreichbare/ungenutzte Erkennung (Stub) |
| SMELL-18 | Feature Envy | Externes-Aufruf-Verhältnis |
| SMELL-19 | Inappropriate Intimacy | Klassengrenzen-überschreitender privater Zugriff (Stub) |
| SMELL-20 | Message Chains | Aufrufketten-Tiefe |
| SMELL-21 | God Object | Zusammengesetzt: LOC + Methoden + Kopplung |
| SMELL-22 | Refused Bequest | Override-to-nothing-Verhältnis (Stub) |
| SMELL-23 | Alternative Classes with Different Interfaces | Schnittstellen-Divergenz (Stub) |

---

## Entwicklungseinrichtung

```bash
# Klonen und kompilieren (erfordert Rust 1.95+)
git clone https://github.com/epicsagas/Episteme.git
cd Episteme
cargo build

# Tests ausführen
cargo test

# Linting
cargo clippy -- -D warnings

# Lokal installieren (importiert Daten und erstellt DB automatisch)
cargo install --path .
epis install --local
```

---

## API-Endpunkte (17)

| Methode | Pfad | Beschreibung |
|---------|------|-------------|
| GET | `/` | Dienstinformationen |
| GET | `/health` | Zustandsprüfung |
| GET | `/live` | Verfügbarkeitsprüfung |
| GET | `/ready` | Bereitschaftsprüfung |
| GET | `/stats` | Graph-Statistiken |
| POST | `/analyze` | Code-Smell-Erkennung |
| POST | `/refactor` | Refactoring-Vorschläge |
| GET | `/search` | Wissenssuche |
| POST | `/search` | Wissenssuche (POST) |
| GET | `/graph/{id}` | Entität abrufen |
| GET | `/graph/{id}/neighbors` | Nachbarn abrufen |
| POST | `/graph/neighbors` | Nachbarn abrufen (POST) |
| POST | `/graph/subgraph` | Subgraph extrahieren |
| GET | `/graph/path` | Kürzester Pfad |
| GET | `/graph/contradictions` | Widersprüche finden |
| POST | `/graph/infer-transitive` | Transitive Relationen ableiten |
| GET | `/metrics` | Prometheus-Metriken |

---

## Zukunfts-Roadmap

- **IDE-Plugins** — VSCode, IntelliJ native Integrationen
- **Benutzerdefinierte Entitäten** — Team-spezifische Muster/Smells hinzufügen
- **Team-Metriken** — Aggregierte Musternutzung über die Organisation
- **Mehrsprachige Dokumentation** — Wissensbasis auf Koreanisch, Japanisch, Chinesisch
- **Interaktive Tutorials** - In-App-geführte Touren für MCP-Tools

---

*Letzte Aktualisierung: 2026-05-03*
