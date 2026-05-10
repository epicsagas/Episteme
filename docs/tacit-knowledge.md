# Tacit Knowledge Architecture

Episteme manages two distinct layers of knowledge: **canonical** (immutable, curated) and **tacit** (mutable, user-contributed). This document describes the two-database architecture, data flow, and the insight lifecycle.

## Overview

| | Canonical Knowledge | Tacit Knowledge (Insights) |
|---|---|---|
| **Storage** | `~/.episteme/db/episteme.db` | `~/.episteme/user_knowledge.db` |
| **Mutability** | Read-only (rebuilt via `epis build`) | Read-write (real-time via MCP) |
| **ID prefix** | `DP-NNN`, `RF-NNN`, `LAW-NNN`, `SMELL-NNN` | `TK-NNN` |
| **Source** | Curated markdown files in `raw/` | MCP `add_insight` tool / CLI `epis insight` |
| **Entities** | 22 patterns, 66 refactorings, 56 laws, 23 smells | Unlimited user insights |

These two databases are physically separate but merged at runtime into a single traversable graph.

## Two-Database Design

```
┌─────────────────────────────────┐     ┌──────────────────────────────┐
│  Canonical DB (episteme.db)     │     │  User Knowledge DB           │
│                                 │     │  (user_knowledge.db)         │
│  ┌───────────┐  ┌────────────┐  │     │  ┌────────────────────────┐  │
│  │  chunks   │  │ embeddings │  │     │  │  user_entities         │  │
│  │  (914)    │  │  (914)     │  │     │  │  (TK-xxx entries)      │  │
│  └───────────┘  └────────────┘  │     │  ├────────────────────────┤  │
│                                 │     │  │  user_relations        │  │
│  Built by: epis build           │     │  ├────────────────────────┤  │
│  Populated from: raw/*.md       │     │  │  user_embeddings       │  │
│                                 │     │  ├────────────────────────┤  │
│  Immutable at runtime           │     │  │  user_entities_fts     │  │
│                                 │     │  │  (FTS5 search index)   │  │
└──────────────┬──────────────────┘     │  ├────────────────────────┤  │
               │                        │  │  insight_seq           │  │
               │                        │  │  (atomic ID counter)   │  │
               │                        │  └────────────────────────┘  │
               │                        │                              │
               │                        │  Written by: MCP add_insight │
               │                        │  Read by: search_insights    │
               │                        └──────────────┬───────────────┘
               │                                       │
               └───────────────┬───────────────────────┘
                               │
                    ┌──────────▼──────────┐
                    │   CompositeGraph    │
                    │   (in-memory merge) │
                    │                     │
                    │  - Unified entity   │
                    │    lookup           │
                    │  - Cross-layer BFS  │
                    │  - Cross-layer      │
                    │    neighbor queries │
                    │                     │
                    │  Serves all MCP     │
                    │  tool requests      │
                    └─────────────────────┘
```

### Why separate databases?

1. **Protection** — User input cannot corrupt the curated canonical knowledge.
2. **Independent lifecycle** — Canonical knowledge updates via the build pipeline; tacit knowledge updates in real-time.
3. **Portability** — Share `user_knowledge.db` across machines or teams without touching the canonical layer.

## CompositeGraph

The `CompositeGraph` struct (in `src/domain/composite_graph.rs`) merges both layers into a single `GraphRepository` interface at startup:

- Loads the canonical `KnowledgeGraph` from `relations.json`
- Opens `user_knowledge.db` via `UserGraphStore`
- Provides unified `get_entity()`, `get_neighbors()`, `find_path()` across both layers
- User operations never modify the canonical graph

### Graceful fallback

If `user_knowledge.db` cannot be opened (missing file, permission error), the system falls back to canonical-only mode. All 6 canonical MCP tools continue working; the 3 tacit knowledge tools return an error.

## User Knowledge Schema

```sql
-- Core entity table
CREATE TABLE user_entities (
    id TEXT PRIMARY KEY,                    -- e.g. "TK-001"
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    author TEXT NOT NULL DEFAULT 'user',
    confidence REAL NOT NULL DEFAULT 0.5,   -- 0.0 to 1.0
    evidence_count INTEGER NOT NULL DEFAULT 0,
    last_validated TEXT NOT NULL DEFAULT '',
    tags TEXT NOT NULL DEFAULT '[]',        -- JSON array
    relations TEXT NOT NULL DEFAULT '{}',   -- JSON: type -> [target_ids]
    created_at TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL DEFAULT '',
    link_provenance TEXT NOT NULL DEFAULT '{}'  -- JSON: entity_id -> metadata
);

-- Explicit relation edges
CREATE TABLE user_relations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_id TEXT NOT NULL,
    relation_type TEXT NOT NULL,
    to_id TEXT NOT NULL,
    UNIQUE(from_id, relation_type, to_id)
);

-- Embedding vectors (f32, little-endian)
CREATE TABLE user_embeddings (
    entity_id TEXT PRIMARY KEY,
    embedding BLOB NOT NULL
);

-- Full-text search index
CREATE VIRTUAL TABLE user_entities_fts USING fts5(
    title, content, tags,
    content=user_entities, content_rowid=rowid
);

-- Atomic ID sequence
CREATE TABLE insight_seq (key TEXT PRIMARY KEY, val INTEGER NOT NULL);
```

## MCP Tools

### add_insight

Creates a `TK-NNN` entity from free text. The system automatically:

1. **Detects canonical entity links** — Two-phase keyword matching (stop-word filtering + composite scoring) finds relevant patterns, laws, and smells.
2. **Checks for duplicates** — Compares against existing insights.
3. **Creates `derives_from` relations** — For high-confidence links (score >= 0.5), automatically links to canonical entities.
4. **Computes correlations** — Finds related insights using Jaccard similarity.

Parameters:
- `text` (required) — Free-text insight content
- `project` (optional) — Project name tag
- `tags` (optional) — Category tags
- `linked_entities` (optional) — Explicit entity IDs to link (e.g. `["DP-005", "SMELL-01"]`)

### search_insights

FTS5 keyword search over user-contributed insights. Returns matching `TK-*` entities with their content and relations.

Parameters:
- `query` (required) — Natural-language search query
- `limit` (optional) — Max results (default 10, max 20)

### confirm_links

Validates or rejects auto-detected links between an insight and canonical entities. Each confirmation:

- Boosts the insight's confidence score (+0.05 per confirmed link, capped at 1.0)
- Records link provenance (source, score, timestamp)
- Supports merge/supersede relations between insights

Parameters:
- `insight_id` (required) — The `TK-NNN` ID
- `accepted` (required) — Entity IDs to confirm as valid links
- `rejected` (optional) — Entity IDs to reject
- `merged_with` (optional) — Target insight ID for merge/supersede

## Insight Lifecycle

```
1. add_insight("마이크로서비스 분리 시 도메인 경계를 먼저 식별하기로 결정")
       │
       ▼
2. Auto-detect links: CONWAY-001 (Conway's Law), DP-026 (Strangler Fig)
       │
       ▼
3. Create TK-001 with derives_from → LAW-017, DP-026
       │
       ▼
4. confirm_links(insight_id="TK-001", accepted=["LAW-017"])
       │
       ▼
5. Confidence boosted: 0.5 → 0.55
       │
       ▼
6. Later: search_insights("마이크로서비스 분리") → returns TK-001
       │
       ▼
7. find_path("TK-001", "SMELL-03") → traverses cross-layer graph
```

## Relation Types

| Relation | Direction | Description |
|----------|-----------|-------------|
| `derives_from` | TK → Canonical | Insight grounded in a canonical entity |
| `applies_to` | TK → Canonical | Insight applies a pattern/law to a specific context |
| `supersedes` | TK → TK | Newer insight replaces an older one |
| `related_to` | TK → TK/Canonical | General semantic connection |

## CLI Usage

```bash
# Add an insight
epis insight add "팀에서 God Class 리팩토링 시 Extract Class보다 Facade Pattern이 효과적이었음"

# Search insights
epis insight search "인증 미들웨어"

# List all insights
epis insight list
```

## Key Source Files

| File | Role |
|------|------|
| `src/domain/composite_graph.rs` | Runtime merge of canonical + user layers |
| `src/adapters/user_graph_store.rs` | SQLite-backed `MutableGraphRepository` |
| `src/server/mcp_insight.rs` | MCP handlers for the 3 tacit knowledge tools |
| `src/adapters/insight_utils.rs` | ID generation, timestamps, text utilities |
| `src/domain/types.rs` | `UserEntity`, `LinkProvenance`, `EntityType::Insight` |
| `src/ports/graph.rs` | `MutableGraphRepository` trait (14 methods) |
