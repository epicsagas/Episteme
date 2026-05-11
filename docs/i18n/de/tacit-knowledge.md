# Architektur des Impliziten Wissens

Episteme verwaltet zwei unterschiedliche Wissensschichten: **kanonisch** (unveränderlich, kuratiert) und **implizit** (veränderlich, benutzergeneriert). Dieses Dokument beschreibt die Zwei-Datenbank-Architektur, den Datenfluss und den Insight-Lebenszyklus.

## Übersicht

| | Kanonisches Wissen | Implizites Wissen (Insights) |
|---|---|---|
| **Speicher** | `~/.episteme/db/episteme.db` | `~/.episteme/user_knowledge.db` |
| **Veränderbarkeit** | Nur-Lese (Wiederaufbau über `epis build`) | Lese-Schreib (Echtzeit über MCP) |
| **ID-Präfix** | `DP-NNN`, `RF-NNN`, `LAW-NNN`, `SMELL-NNN` | `TK-NNN` |
| **Quelle** | Kuratierte Markdown-Dateien in `raw/` | MCP `add_insight`-Tool / CLI `epis insight` |
| **Entitäten** | 22 Muster, 66 Refactorings, 56 Gesetze, 23 Smells | Unbegrenzte Benutzer-Insights |

Diese beiden Datenbanken sind physisch getrennt, werden aber zur Laufzeit zu einem einzigen begehbaren Graphen zusammengeführt.

## Zwei-Datenbank-Design

```
┌─────────────────────────────────┐     ┌──────────────────────────────┐
│  Kanonische DB (episteme.db)    │     │  Benutzer-Wissens-DB         │
│                                 │     │  (user_knowledge.db)         │
│  ┌───────────┐  ┌────────────┐  │     │  ┌────────────────────────┐  │
│  │  chunks   │  │ embeddings │  │     │  │  user_entities         │  │
│  │  (914)    │  │  (914)     │  │     │  │  (TK-xxx-Einträge)     │  │
│  └───────────┘  └────────────┘  │     │  ├────────────────────────┤  │
│                                 │     │  │  user_relations        │  │
│  Erstellt von: epis build       │     │  ├────────────────────────┤  │
│  Befüllt aus: raw/*.md          │     │  │  user_embeddings       │  │
│                                 │     │  ├────────────────────────┤  │
│  Unveränderlich zur Laufzeit    │     │  │  user_entities_fts     │  │
│                                 │     │  │  (FTS5-Suchindex)      │  │
└──────────────┬──────────────────┘     │  ├────────────────────────┤  │
               │                        │  │  insight_seq           │  │
               │                        │  │  (atomarer ID-Zähler)  │  │
               │                        │  └────────────────────────┘  │
               │                        │                              │
               │                        │  Geschrieben von: MCP add_insight │
               │                        │  Gelesen von: search_insights    │
               │                        └──────────────┬───────────────┘
               │                                       │
               └───────────────┬───────────────────────┘
                               │
                    ┌──────────▼──────────┐
                    │   CompositeGraph    │
                    │   (In-Memory-Merge) │
                    │                     │
                    │  - Einheitliche     │
                    │    Entitätsabfrage  │
                    │  - Cross-Layer-BFS  │
                    │  - Cross-Layer      │
                    │    Nachbarabfragen  │
                    │                     │
                    │  Bedient alle MCP   │
                    │  Tool-Anfragen      │
                    └─────────────────────┘
```

### Warum getrennte Datenbanken?

1. **Schutz** — Benutzereingaben können das kuratierte kanonische Wissen nicht beschädigen.
2. **Unabhängiger Lebenszyklus** — Kanonisches Wissen wird über die Build-Pipeline aktualisiert; implizites Wissen wird in Echtzeit aktualisiert.
3. **Portabilität** — Teilen Sie `user_knowledge.db` über Maschinen oder Teams hinweg, ohne die kanonische Schicht zu berühren.

## CompositeGraph

Die `CompositeGraph`-Struktur (in `src/domain/composite_graph.rs`) führt beide Schichten beim Start zu einer einzigen `GraphRepository`-Schnittstelle zusammen:

- Lädt den kanonischen `KnowledgeGraph` aus `relations.json`
- Öffnet `user_knowledge.db` über `UserGraphStore`
- Bietet einheitliches `get_entity()`, `get_neighbors()`, `find_path()` über beide Schichten
- Benutzeroperationen verändern niemals den kanonischen Graphen

### Graceful Fallback

Wenn `user_knowledge.db` nicht geöffnet werden kann (fehlende Datei, Berechtigungsfehler), fällt das System auf den rein kanonischen Modus zurück. Alle 6 kanonischen MCP-Tools funktionieren weiterhin; die 3 Tools für implizites Wissen geben einen Fehler zurück.

## Benutzer-Wissens-Schema

```sql
-- Kern-Entitätstabelle
CREATE TABLE user_entities (
    id TEXT PRIMARY KEY,                    -- z.B. "TK-001"
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    author TEXT NOT NULL DEFAULT 'user',
    confidence REAL NOT NULL DEFAULT 0.5,   -- 0.0 bis 1.0
    evidence_count INTEGER NOT NULL DEFAULT 0,
    last_validated TEXT NOT NULL DEFAULT '',
    tags TEXT NOT NULL DEFAULT '[]',        -- JSON-Array
    relations TEXT NOT NULL DEFAULT '{}',   -- JSON: Typ -> [Ziel-IDs]
    created_at TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL DEFAULT '',
    link_provenance TEXT NOT NULL DEFAULT '{}'  -- JSON: entity_id -> Metadaten
);

-- Explizite Relationskanten
CREATE TABLE user_relations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_id TEXT NOT NULL,
    relation_type TEXT NOT NULL,
    to_id TEXT NOT NULL,
    UNIQUE(from_id, relation_type, to_id)
);

-- Embedding-Vektoren (f32, Little-Endian)
CREATE TABLE user_embeddings (
    entity_id TEXT PRIMARY KEY,
    embedding BLOB NOT NULL
);

-- Volltext-Suchindex
CREATE VIRTUAL TABLE user_entities_fts USING fts5(
    title, content, tags,
    content=user_entities, content_rowid=rowid
);

-- Atomare ID-Sequenz
CREATE TABLE insight_seq (key TEXT PRIMARY KEY, val INTEGER NOT NULL);
```

## MCP-Tools

### add_insight

Erstellt eine `TK-NNN`-Entität aus Freitext. Das System führt automatisch Folgendes durch:

1. **Erkennt kanonische Entitätsverknüpfungen** — Zweiphasige Schlüsselwortübereinstimmung (Stoppwort-Filterung + zusammengesetzte Bewertung) findet relevante Muster, Gesetze und Smells.
2. **Prüft auf Duplikate** — Vergleicht mit vorhandenen Insights.
3. **Erstellt `derives_from`-Relationen** — Für Verknüpfungen mit hoher Konfidenz (Score >= 0.5) wird automatisch mit kanonischen Entitäten verknüpft.
4. **Berechnet Korrelationen** — Findet verwandte Insights mittels Jaccard-Ähnlichkeit.

Parameter:
- `text` (erforderlich) — Freitext-Insight-Inhalt
- `project` (optional) — Projektname-Tag
- `tags` (optional) — Kategorie-Tags
- `linked_entities` (optional) — Explizite Entitäts-IDs zur Verknüpfung (z.B. `["DP-005", "SMELL-01"]`)

### search_insights

FTS5-Schlüsselwortsuche über benutzergenerierte Insights. Gibt übereinstimmende `TK-*`-Entitäten mit ihren Inhalten und Relationen zurück.

Parameter:
- `query` (erforderlich) — Suchanfrage in natürlicher Sprache
- `limit` (optional) — Maximale Ergebnisse (Standard 10, maximal 20)

### confirm_links

Validiert oder lehnt automatisch erkannte Verknüpfungen zwischen einem Insight und kanonischen Entitäten ab. Jede Bestätigung:

- Erhöht den Konfidenzwert des Insights (+0,05 pro bestätigter Verknüpfung, maximal 1,0)
- Erfasst die Verknüpfungsprovenienz (Quelle, Score, Zeitstempel)
- Unterstützt Merge/Supersede-Relationen zwischen Insights

Parameter:
- `insight_id` (erforderlich) — Die `TK-NNN`-ID
- `accepted` (erforderlich) — Entitäts-IDs, die als gültige Verknüpfungen bestätigt werden
- `rejected` (optional) — Abzulehnende Entitäts-IDs
- `merged_with` (optional) — Ziel-Insight-ID für Merge/Supersede

## Insight-Lebenszyklus

```
1. add_insight("마이크로서비스 분리 시 도메인 경계를 먼저 식별하기로 결정")
       │
       ▼
2. Automatische Link-Erkennung: CONWAY-001 (Conway's Law), DP-026 (Strangler Fig)
       │
       ▼
3. TK-001 erstellen mit derives_from → LAW-017, DP-026
       │
       ▼
4. confirm_links(insight_id="TK-001", accepted=["LAW-017"])
       │
       ▼
5. Konfidenz erhöht: 0.5 → 0.55
       │
       ▼
6. Später: search_insights("마이크로서비스 분리") → gibt TK-001 zurück
       │
       ▼
7. find_path("TK-001", "SMELL-03") → durchläuft Cross-Layer-Graph
```

## Relationstypen

| Relation | Richtung | Beschreibung |
|----------|----------|-------------|
| `derives_from` | TK → Kanonisch | Insight basiert auf einer kanonischen Entität |
| `applies_to` | TK → Kanonisch | Insight wendet ein Muster/Gesetz auf einen bestimmten Kontext an |
| `supersedes` | TK → TK | Neuerer Insight ersetzt einen älteren |
| `related_to` | TK → TK/Kanonisch | Allgemeine semantische Verbindung |

## CLI-Verwendung

```bash
# Insight hinzufügen
epis insight add "팀에서 God Class 리팩토링 시 Extract Class보다 Facade Pattern이 효과적이었음"

# Insights durchsuchen
epis insight search "인증 미들웨어"

# Alle Insights auflisten
epis insight list
```

## Wichtige Quelldateien

| Datei | Rolle |
|-------|-------|
| `src/domain/composite_graph.rs` | Laufzeit-Zusammenführung von kanonischer + Benutzerschicht |
| `src/adapters/user_graph_store.rs` | SQLite-gestütztes `MutableGraphRepository` |
| `src/server/mcp_insight.rs` | MCP-Handler für die 3 Tools für implizites Wissen |
| `src/adapters/insight_utils.rs` | ID-Generierung, Zeitstempel, Text-Dienstprogramme |
| `src/domain/types.rs` | `UserEntity`, `LinkProvenance`, `EntityType::Insight` |
| `src/ports/graph.rs` | `MutableGraphRepository`-Trait (14 Methoden) |
