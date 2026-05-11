# Alcove-Ökosystem — Architektur- und Fähigkeitsanalyse

> Ein detaillierter Vergleich von Epistemes Tacit-Knowledge-Schicht (TK-*) und dem Alcove-Dokumentationsökosystem, der Speichermodelle, Suchfähigkeiten, Lebenszyklusverwaltung und Anwendungsfälle abdeckt.

---

## 1. Architekturübersicht

### Episteme Tacit Knowledge (TK-*)

| Aspekt | Detail |
|--------|--------|
| **Speicherung** | SQLite-Einzeldatei (`~/.episteme/user_knowledge.db`) |
| **Schema** | 5 Tabellen: `user_entities`, `user_relations`, `user_embeddings`, `user_entities_fts` (FTS5-virtuell), `insight_seq` |
| **Einheit** | Eine Erkenntnis = eine `UserEntity`-Zeile (TK-xxx-ID) |
| **Graph** | Zur Laufzeit über `CompositeGraph` mit dem kanonischen Graph zusammengeführt — ermöglicht cross-schicht Pfadtraversierung (TK-001 → DP-005 → SMELL-01) |
| **Nebenläufigkeit** | `Mutex<Connection>` + WAL-Modus für gleichzeitigen MCP + CLI-Zugriff |

### Alcove-Dokumentationssystem

| Aspekt | Detail |
|--------|--------|
| **Speicherung** | Markdown-Dateien im Dateisystem + Tantivy BM25-Index + sqlite-vec-Embeddings |
| **Struktur** | 3-stufige Klassifikation: Core (7), Supplementary (19), Public (15) Dateien pro Projekt |
| **Einheit** | Eine strukturierte Markdown-Datei (PRD, ARCHITECTURE, DECISIONS usw.) |
| **Graph** | Wikilink + dateipfadbasierte lose Verbindungen |
| **Nebenläufigkeit** | Dateibasierte Sperre (`.index_lock`) je Dokument-Root, pro-Vault-Indexisolation |
| **Vaults** | 3 Symlinks zu Obsidian PARA-Ordnern: areas (8 Dokumente), resources (71), zettelkasten (17) |

---

## 2. Speichermodellvergleich

### Episteme TK-*-Schema

```sql
-- Kerntabelle
CREATE TABLE user_entities (
    id TEXT PRIMARY KEY,           -- TK-001, TK-002, ...
    title TEXT,                    -- Auto: erste Zeile, max. 80 Zeichen
    content TEXT,                  -- Freitext (keine maximale Länge)
    author TEXT DEFAULT 'user',
    confidence REAL DEFAULT 0.5,   -- +0.05 pro bestätigtem Link, max. 1.0
    evidence_count INTEGER DEFAULT 0,
    last_validated TEXT DEFAULT '',
    tags TEXT DEFAULT '[]',        -- JSON-Array
    relations TEXT DEFAULT '{}',   -- JSON HashMap<relation_type, Vec<entity_id>>
    link_provenance TEXT DEFAULT '{}',
    created_at TEXT,
    updated_at TEXT
);

-- Normalisierte Relationen (derives_from, applies_to, supersedes)
CREATE TABLE user_relations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_id TEXT,
    relation_type TEXT,
    to_id TEXT,
    UNIQUE(from_id, relation_type, to_id)
);

-- FTS5-Volltextsuche
CREATE VIRTUAL TABLE user_entities_fts USING fts5(title, content, tags, content=user_entities);
```

### Alcove-Dateistruktur

```
~/.alcove/
  config.toml                    # Globale Konfiguration (docs_root, core/team/public Dateilisten, Embedding-Modell)
  docs -> Symlink                # → Obsidian/SecondBrain/99-Archives/projects
  vaults/
    areas -> Symlink             # → Obsidian/02-Areas (8 Dokumente)
    resources -> Symlink         # → Obsidian/03-Resources (71 Dokumente)
    zettelkasten -> Symlink      # → Obsidian/10-Zettelkasten (17 Dokumente)
  models/                        # Gecachte ONNX-Embedding-Modelle
  logs/

<docs_root>/<project>/
  .alcove/
    index/                       # Tantivy BM25-Indexdateien
    index_meta.json              # Datei-Fingerabdrücke (mtime + size)
    vectors.db                   # sqlite-vec-Embeddings
  PRD.md                         # Produktanforderungen
  ARCHITECTURE.md                # Systemdesign
  PROGRESS.md                    # Meilensteine & Status
  DECISIONS.md                   # Architecture Decision Records
  CONVENTIONS.md                 # Coding-Standards
  SECRETS_MAP.md                 # Umgebungsvariablen & Secrets
  DEBT.md                        # Technische-Schulden-Register
```

---

## 3. Wissenscharakter

| Dimension | Episteme TK-* | Alcove |
|-----------|---------------|--------|
| **Typ** | Augenblickliche Erkenntnisse, Lessons Learned, Teamentscheidungen | Strukturierte Projektdokumentation (Anforderungen, Architektur, Entscheidungen) |
| **Mutabilität** | Mutierbar (SQLite CRUD) | Mutierbar (Dateibearbeitung + Index-Neuaufbau) |
| **Quelle** | Benutzerbeitragter Freitext | Benutzer geschrieben + Agent aus Vorlagen generiert |
| **Autorität** | Persönliche/Team-Beobachtung | Teammandat / Organisationsrichtlinie |
| **Granularität** | Atomar (eine Erkenntnis pro Eintrag) | Sectioniert (mehrere ADRs pro DECISIONS.md) |
| **Verlinkung** | Automatisch zu kanonischen Entitäten erkannt (Keyword-Scoring) | Manuelle Wikilinks + Markdown-Links |
| **Versionierung** | Keine (nur SQLite) | Git-basiert (Datei = Source of Truth) |

### Erkenntnis-Lebenszyklus (Episteme TK-*)

```
add_insight(text, tags?, project?, linked_entities?)
  │
  ├── TK-xxx-ID generieren (atomare Sequenz)
  ├── detect_canonical_links() — Keyword-Matching → Top 5 kanonische Entitäten
  │     Score >= 0.5 → Auto-Link (derives_from)
  │     Score < 0.5 → Vorgeschlagener Link
  ├── FTS5-Duplikaterkennung → DuplicateCandidate[]
  ├── In SQLite + In-Memory-Cache persistieren
  └── Rückgabe: { id, auto_links, suggested_links, duplicates, confidence }

confirm_links(id, accepted[], rejected[])
  │
  ├── derives_from/applies_to-Relationen hinzufügen
  ├── link_provenance-Quelle auf "manual" aktualisieren
  ├── Konfidenz erhöhen (+0.05 pro Link, max. 1.0)
  └── Updates persistieren

search_insights(query, limit?)
  │
  └── FTS5 MATCH-Abfrage → rankierte Ergebnisse
```

### Dokument-Lebenszyklus (Alcove)

```
init_project(project_name, project_path?)
  │
  ├── 7 Core-Dokumente aus Vorlagen erstellen (PRD, ARCHITECTURE, ...)
  ├── Optional öffentliche Dokumente erstellen (README, CHANGELOG, ...)
  └── Suchindex neu aufbauen

validate_docs()
  │
  ├── Erforderliche Dateiexistenz prüfen
  ├── Vorlagen-Platzhalter prüfen (TODO, FIXME)
  ├── Erforderliche Abschnittsüberschriften prüfen
  ├── Minimale Listenelementanzahl prüfen
  └── Rückgabe: pass/warn/fail pro Datei

lint_project()
  │
  ├── Defekte [[wikilinks]] und Markdown-Links erkennen
  ├── Verwaiste Dateien finden (von keinem Dokument verlinkt)
  ├── Veraltete Marker finden (WIP, TODO, FIXME, DRAFT, DEPRECATED)
  └── Veraltete Jahresreferenzen finden (2+ Jahre alt)

audit_project()
  │
  ├── Privates Doc-Repo nach fehlenden Pflichtdokumenten scannen
  ├── Öffentliches Projekt-Repo nach exponierten internen Dokumenten scannen
  ├── Dateien in Stufen klassifizieren
  └── Rückgabe: suggested_actions[]
```

---

## 4. Suchfähigkeiten

| Fähigkeit | Episteme TK-* | Alcove |
|-----------|---------------|--------|
| **Engine** | FTS5 (Keyword-Match) | Tantivy BM25 + sqlite-vec Cosine Similarity |
| **Fusion** | Keine | RRF (Reciprocal Rank Fusion, k=60) |
| **CJK** | Keine spezielle Unterstützung | NgramTokenizer (min=2, max=3) |
| **Chunking** | N/A (eine Zeile = eine Erkenntnis) | 200–500 Zeichen Chunks |
| **Inkrementell** | N/A (einzelne Tabelle) | mtime + size Fingerabdruckvergleich |
| **Vektorsuche** | Schema existiert (`user_embeddings`), aber **nicht angeschlossen** | Voll funktionsfähig (MultilingualE5Small, 384d) |
| **Gültigkeitsbereich** | Einzelne Datenbank | Pro-Projekt oder global (projektübergreifend) |
| **Fallback** | Keiner | grep-Substring-Match wenn kein Index vorhanden |

---

## 5. Funktionsvollständigkeit

| Funktion | Episteme TK-* | Alcove |
|----------|---------------|--------|
| Erstellen | `add_insight` | `init_project`, Dateibearbeitung |
| Lesen | `search_insights` (nur Suche, kein Abruf nach ID) | `get_doc_file`, `search_project_docs` |
| Aktualisieren | Nicht über MCP verfügbar | Direkte Dateibearbeitung + `rebuild_index` |
| Löschen | Nicht über MCP verfügbar | Datei löschen + `rebuild_index` |
| Validierung | Keine | `validate_docs`, `lint_project` |
| Audit | Keine | `audit_project` (öffentliche/private Trennung) |
| Backup | Keine | `backup_vault` (Git-Commit-Snapshot) |
| Import | Keine | `promote_document` (Obsidian → Doc-Repo) |
| Richtlinie | Keine | `policy.toml` mit Enforcement-Leveln |
| Vorlagen | Keine | 7 Core + 19 Supplementary + 15 Public |

---

## 6. Alcove-Vault-System

Drei Vaults, per Symlink mit der Obsidian PARA-Struktur verbunden:

| Vault | Ziel | Dokumente | Zweck |
|-------|------|-----------|-------|
| `areas` | `02-Areas` | 8 | Domänenbereiche: MCP-Agenten, DevOps, Rust, LLM/RAG, Open Source |
| `resources` | `03-Resources` | 71 | Referenz: AWS, Gesetze des Software Engineering, technische Dokumentation |
| `zettelkasten` | `10-Zettelkasten` | 17 | Atomare Notizen: KI-Architektur, BM25, Knowledge Graphs, Rust-Patterns |

Jeder Vault verfügt über unabhängige:
- BM25-Indizes (Tantivy)
- Vektordatenbanken (sqlite-vec)
- Datei-Fingerabdruck-Tracking (`index_meta.json`)
- Cache-Isolation (separate `OnceLock<Mutex<HashMap>>`)

---

## 7. Alcove-Konfigurationssystem

### Global: `~/.alcove/config.toml`

```toml
docs_root = "/path/to/Obsidian/SecondBrain/99-Archives/projects"

[core]
files = ["PRD.md", "ARCHITECTURE.md", "PROGRESS.md", "DECISIONS.md",
         "CONVENTIONS.md", "SECRETS_MAP.md", "DEBT.md"]

[team]
files = ["ENV_SETUP.md", "ONBOARDING.md", "DATA_MODEL.md", "SCHEMA.md",
         "DEPLOYMENT.md", "RUNBOOK.md", "PLAYBOOK.md", "MONITORING.md", ...]  # 19 Dateien

[public]
files = ["README.md", "CHANGELOG.md", "CONTRIBUTING.md", "SECURITY.md", ...]  # 15 Dateien

[embedding]
model = "MultilingualE5Small"
auto_download = true
enabled = true
```

### Pro-Projekt: `alcove.toml`

Überschreibt globale Standardwerte für: `diagram_format`, `core_files`, `team_files`, `public_files`.

### Richtlinie: `policy.toml`

Definiert:
- `enforce`-Level: `strict` | `warn` | `off`
- Erforderliche Dokumente mit Abschnittsüberschriften und minimalen Elementanzahlen
- Namenskonventionen (`UPPER_SNAKE`, `lower_snake`, `kebab`, `free`)
- Priorität: Projekt > Team | integrierte Standardwerte

---

## 8. Anwendungsfall-Entscheidungsmatrix

| Situation | Empfohlenes Tool | Begründung |
|-----------|-----------------|------------|
| "Eine Lesson Learned aus einem Produktionsvorfall festhalten" | **Episteme TK-*** | Auto-Links zu relevanten Smells/Gesetzen für zukünftige Querverweise |
| "Dokumentation für ein neues Projekt starten" | **Alcove** `init_project` | 7 Core-Vorlagen automatisch generiert |
| "Prüfen, ob Dokumente veraltet sind" | **Alcove** `lint_project` | Erkennt automatisch WIP/TODO/DEPRECATED/veraltete Daten |
| "Herausfinden, was das Team über Auth-Middleware entschieden hat" | **Alcove** `search_project_docs` | Durchsucht strukturierte DECISIONS.md mit BM25 + Vektor |
| "Code-Smells in einem Modul erkennen" | **Episteme** `analyze_code` | Pattern/regex-basierte Smell-Erkennung |
| "Sicherstellen, dass PRD alle erforderlichen Abschnitte hat" | **Alcove** `validate_docs` | Richtlinienbasierte Abschnitts- und Elementanzahlvalidierung |
| "Eine Erkenntnis mit dem Strategy Pattern verknüpfen" | **Episteme** `confirm_links` | Erstellt `derives_from`-Kante zur kanonischen Entität |
| "Obsidian-Notizen für Agentenzugriff importieren" | **Alcove** `promote_document` | Importiert in Doc-Repo mit automatischer Projekteingrenzung |
| "Beziehung zwischen SRP und Extract Class finden" | **Episteme** `find_path` | Multi-Hop-Graphtraversierung über Entitätstypen |
| "Projektdokumentationsstatus sichern" | **Alcove** `backup_vault` | Git-Commit-Snapshot mit Zeitstempel |
| "Auf exponierte interne Dokumente im öffentlichen Repo prüfen" | **Alcove** `audit_project` | Scannt sowohl private als auch öffentliche Speicherorte |
| "Priorisierte Refactoring-Vorschläge für Code erhalten" | **Episteme** `suggest_refactorings` | Verbundscoring: Schweregrad x Aufwand x Prinzipienausrichtung |

---

## 9. Komplementäre Rollen

```
Episteme TK-*                     Alcove
"Welches universelle Prinzip      "Was hat unser Team
 gilt hier?"                       darüber entschieden?"

 Augenblickliche Erkenntnis ←────────────→ Strukturierte Entscheidungsaufzeichnung
 Keyword-Auto-Verlinkung               Vorlagenbasiertes Gerüst
 Cross-Schicht-Graphtraversierung      Projektübergreifende Dokumentensuche
 Code-Analyse → Smell-Erkennung        Dok-Analyse → Veraltungs-Erkennung
```

**Wenn beide aktiv sind**: Episteme liefert das universelle "Warum" (Gesetze, Patterns), Alcove das projektspezifische "Was haben wir entschieden" (ADRs, Konventionen). Agenten sollten beide Quellen zitieren, wobei Alcove Vorrang hat, wenn Teamregeln mit allgemeiner Anleitung kollidieren.

---

## 10. Skalierung & Performance

| Metrik | Episteme TK-* | Alcove |
|--------|---------------|--------|
| **Auslegungskapazität** | Hunderte von Erkenntnissen | ~10.000 Dateien |
| **Suchlatenz** | FTS5 sofort (In-Memory) | BM25 < 500ms für Überblick |
| **Token-Effizienz** | Eine Erkenntnis pro Ergebnis | Top-5 Chunks ~1.5k Tokens (vs. ~8k für grep) |
| **Index-Neuaufbau** | Nicht erforderlich (FTS5-Trigger) | Inkrementell: nur geänderte Dateien |
| **Modellgröße** | N/A (nicht angeschlossen) | 15MB (ArcticEmbedXS) bis 2.3GB (BGE-M3) |

---

*Siehe auch: [Alcove-Integrationsleitfaden](./alcove-integration.md) für Verwendungsmuster und Workflow-Beispiele.*
