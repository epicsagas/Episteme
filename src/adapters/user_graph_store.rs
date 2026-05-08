use std::collections::HashMap;
use std::sync::Mutex;

use rusqlite::{Connection, params};

use crate::domain::types::{CorrelationScore, Entity, GraphEdge, UserEntity};
use crate::ports::graph::MutableGraphRepository;

// ---------------------------------------------------------------------------
// Helper for neighbor queries (avoids closure type mismatch)
// ---------------------------------------------------------------------------

fn query_neighbors(
    conn: &Connection,
    base_sql: &str,
    entity_id: &str,
    relation_type: Option<&str>,
    out: &mut Vec<String>,
) {
    let sql = if relation_type.is_some() {
        format!("{base_sql} AND relation_type=?2")
    } else {
        base_sql.to_owned()
    };
    let Ok(mut stmt) = conn.prepare(&sql) else {
        return;
    };
    let rows: Vec<String> = if let Some(rt) = relation_type {
        stmt.query_map(params![entity_id, rt], |row| row.get::<_, String>(0))
            .ok()
            .map(|r| r.filter_map(|v| v.ok()).collect())
            .unwrap_or_default()
    } else {
        stmt.query_map(params![entity_id], |row| row.get::<_, String>(0))
            .ok()
            .map(|r| r.filter_map(|v| v.ok()).collect())
            .unwrap_or_default()
    };
    out.extend(rows);
}

// ---------------------------------------------------------------------------
// Schema initialization
// ---------------------------------------------------------------------------

fn init_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        PRAGMA journal_mode=WAL;

        CREATE TABLE IF NOT EXISTS user_entities (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            author TEXT NOT NULL DEFAULT 'user',
            confidence REAL NOT NULL DEFAULT 0.5,
            evidence_count INTEGER NOT NULL DEFAULT 0,
            last_validated TEXT NOT NULL DEFAULT '',
            tags TEXT NOT NULL DEFAULT '[]',
            relations TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL DEFAULT '',
            updated_at TEXT NOT NULL DEFAULT ''
        );

        CREATE TABLE IF NOT EXISTS user_relations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            from_id TEXT NOT NULL,
            relation_type TEXT NOT NULL,
            to_id TEXT NOT NULL,
            UNIQUE(from_id, relation_type, to_id),
            FOREIGN KEY (from_id) REFERENCES user_entities(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS user_embeddings (
            entity_id TEXT PRIMARY KEY,
            embedding BLOB NOT NULL,
            FOREIGN KEY (entity_id) REFERENCES user_entities(id) ON DELETE CASCADE
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS user_entities_fts USING fts5(
            title, content, tags, content=user_entities, content_rowid=rowid
        );

        CREATE INDEX IF NOT EXISTS idx_user_relations_from ON user_relations(from_id);
        CREATE INDEX IF NOT EXISTS idx_user_relations_to ON user_relations(to_id);

        -- Sequence counter for atomic TK-xxx ID generation
        CREATE TABLE IF NOT EXISTS insight_seq (key TEXT PRIMARY KEY, val INTEGER NOT NULL);
        INSERT OR IGNORE INTO insight_seq (key, val) VALUES ('tk', 0);

        -- FTS5 content-sync triggers (required for content= tables)
        CREATE TRIGGER IF NOT EXISTS user_entities_ai AFTER INSERT ON user_entities BEGIN
            INSERT INTO user_entities_fts(rowid, title, content, tags)
            VALUES (new.rowid, new.title, new.content, new.tags);
        END;

        CREATE TRIGGER IF NOT EXISTS user_entities_ad AFTER DELETE ON user_entities BEGIN
            INSERT INTO user_entities_fts(user_entities_fts, rowid, title, content, tags)
            VALUES ('delete', old.rowid, old.title, old.content, old.tags);
        END;

        CREATE TRIGGER IF NOT EXISTS user_entities_au AFTER UPDATE ON user_entities BEGIN
            INSERT INTO user_entities_fts(user_entities_fts, rowid, title, content, tags)
            VALUES ('delete', old.rowid, old.title, old.content, old.tags);
            INSERT INTO user_entities_fts(rowid, title, content, tags)
            VALUES (new.rowid, new.title, new.content, new.tags);
        END;
        ",
    )
    .map_err(|e| format!("user graph schema init: {e}"))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// UserGraphStore
// ---------------------------------------------------------------------------

/// Convert a [`UserEntity`] to a canonical [`Entity`] for unified graph operations.
///
/// This function lives in the adapter layer because it uses `serde_json::json!`
/// to construct the `source` metadata, keeping the domain free of direct
/// `serde_json` usage.
pub fn user_entity_to_entity(ue: &UserEntity) -> Entity {
    Entity {
        id: ue.id.clone(),
        r#type: "insight".to_owned(),
        title: ue.title.clone(),
        description: ue.content.clone(),
        name: ue.title.clone(),
        category: String::new(),
        tags: ue.tags.clone(),
        relations: ue.relations.clone(),
        context: {
            let mut ctx = HashMap::new();
            ctx.insert("author".to_owned(), vec![ue.author.clone()]);
            ctx.insert(
                "confidence".to_owned(),
                vec![format!("{:.2}", ue.confidence)],
            );
            ctx.insert(
                "evidence_count".to_owned(),
                vec![ue.evidence_count.to_string()],
            );
            ctx.insert("last_validated".to_owned(), vec![ue.last_validated.clone()]);
            ctx
        },
        file_path: String::new(),
        source: serde_json::json!({
            "source": "user",
            "confidence": ue.confidence,
            "author": ue.author,
        }),
    }
}

pub struct UserGraphStore {
    conn: Mutex<Connection>,
}

impl UserGraphStore {
    pub fn open(path: &std::path::Path) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| format!("open user graph: {e}"))?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")
            .map_err(|e| format!("enable FK: {e}"))?;
        init_schema(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn open_in_memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory().map_err(|e| format!("open in-memory: {e}"))?;
        init_schema(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Atomically generate the next TK-xxx ID using a SQLite sequence counter.
    /// Syncs the counter with any manually-added entities to prevent collisions.
    /// Safe under concurrent writes (MCP server + CLI simultaneously).
    pub fn next_insight_id_atomic(&self) -> Result<String, String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("mutex poisoned: {e}"))?;

        // Sync: ensure sequence is at least as high as any existing TK-xxx entity
        let max_existing: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(CAST(SUBSTR(id, 4) AS INTEGER)), 0) \
                 FROM user_entities WHERE id LIKE 'TK-%'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0);
        if max_existing > 0 {
            conn.execute(
                "UPDATE insight_seq SET val = MAX(val, ?1) WHERE key = 'tk'",
                params![max_existing],
            )
            .map_err(|e| format!("sync sequence: {e}"))?;
        }

        // Atomic increment and return
        let next: i64 = conn
            .query_row(
                "UPDATE insight_seq SET val = val + 1 WHERE key = 'tk' RETURNING val",
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("next insight id: {e}"))?;
        Ok(format!("TK-{:03}", next))
    }
}

// ---------------------------------------------------------------------------
// Row mapping helpers
// ---------------------------------------------------------------------------

fn row_to_user_entity(row: &rusqlite::Row<'_>) -> rusqlite::Result<UserEntity> {
    let tags_str: String = row.get(7)?;
    let rels_str: String = row.get(8)?;
    Ok(UserEntity {
        id: row.get(0)?,
        title: row.get(1)?,
        content: row.get(2)?,
        author: row.get(3)?,
        confidence: row.get(4)?,
        evidence_count: row.get(5)?,
        last_validated: row.get(6)?,
        tags: serde_json::from_str(&tags_str).unwrap_or_default(),
        relations: serde_json::from_str(&rels_str).unwrap_or_default(),
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

// ---------------------------------------------------------------------------
// MutableGraphRepository implementation
// ---------------------------------------------------------------------------

impl MutableGraphRepository for UserGraphStore {
    fn add_entity(&self, entity: UserEntity) -> Result<(), String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("mutex poisoned: {e}"))?;
        let tags = serde_json::to_string(&entity.tags).unwrap_or_else(|_| "[]".to_owned());
        let rels = serde_json::to_string(&entity.relations).unwrap_or_else(|_| "{}".to_owned());
        conn.execute(
            "INSERT INTO user_entities (id, title, content, author, confidence, evidence_count, last_validated, tags, relations, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                entity.id, entity.title, entity.content, entity.author,
                entity.confidence, entity.evidence_count, entity.last_validated,
                tags, rels, entity.created_at, entity.updated_at,
            ],
        )
        .map_err(|e| format!("insert user entity: {e}"))?;

        // Insert relations
        for (rel_type, targets) in &entity.relations {
            for target in targets {
                conn.execute(
                    "INSERT OR IGNORE INTO user_relations (from_id, relation_type, to_id) VALUES (?1, ?2, ?3)",
                    params![entity.id, rel_type, target],
                )
                .map_err(|e| format!("insert relation: {e}"))?;
            }
        }
        Ok(())
    }

    fn update_entity(&self, id: &str, entity: UserEntity) -> Result<(), String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("mutex poisoned: {e}"))?;
        let tags = serde_json::to_string(&entity.tags).unwrap_or_else(|_| "[]".to_owned());
        let rels = serde_json::to_string(&entity.relations).unwrap_or_else(|_| "{}".to_owned());
        let rows = conn
            .execute(
                "UPDATE user_entities SET title=?1, content=?2, author=?3, confidence=?4,
                 evidence_count=?5, last_validated=?6, tags=?7, relations=?8, updated_at=?9
                 WHERE id=?10",
                params![
                    entity.title,
                    entity.content,
                    entity.author,
                    entity.confidence,
                    entity.evidence_count,
                    entity.last_validated,
                    tags,
                    rels,
                    entity.updated_at,
                    id,
                ],
            )
            .map_err(|e| format!("update user entity: {e}"))?;
        if rows == 0 {
            return Err(format!("entity not found: {id}"));
        }

        // Re-sync relations: delete old, insert new
        conn.execute("DELETE FROM user_relations WHERE from_id=?1", params![id])
            .map_err(|e| format!("delete old relations: {e}"))?;
        for (rel_type, targets) in &entity.relations {
            for target in targets {
                conn.execute(
                    "INSERT OR IGNORE INTO user_relations (from_id, relation_type, to_id) VALUES (?1, ?2, ?3)",
                    params![id, rel_type, target],
                )
                .map_err(|e| format!("insert relation: {e}"))?;
            }
        }
        Ok(())
    }

    fn remove_entity(&self, id: &str) -> Result<(), String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("mutex poisoned: {e}"))?;
        conn.execute(
            "DELETE FROM user_relations WHERE from_id=?1 OR to_id=?1",
            params![id],
        )
        .map_err(|e| format!("delete relations: {e}"))?;
        conn.execute(
            "DELETE FROM user_embeddings WHERE entity_id=?1",
            params![id],
        )
        .map_err(|e| format!("delete embedding: {e}"))?;
        let rows = conn
            .execute("DELETE FROM user_entities WHERE id=?1", params![id])
            .map_err(|e| format!("delete entity: {e}"))?;
        if rows == 0 {
            return Err(format!("entity not found: {id}"));
        }
        Ok(())
    }

    fn add_relation(&self, from: &str, relation: &str, to: &str) -> Result<(), String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("mutex poisoned: {e}"))?;
        conn.execute(
            "INSERT OR IGNORE INTO user_relations (from_id, relation_type, to_id) VALUES (?1, ?2, ?3)",
            params![from, relation, to],
        )
        .map_err(|e| format!("insert relation: {e}"))?;
        Ok(())
    }

    fn remove_relation(&self, from: &str, relation: &str, to: &str) -> Result<(), String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("mutex poisoned: {e}"))?;
        conn.execute(
            "DELETE FROM user_relations WHERE from_id=?1 AND relation_type=?2 AND to_id=?3",
            params![from, relation, to],
        )
        .map_err(|e| format!("delete relation: {e}"))?;
        Ok(())
    }

    fn get_user_entity(&self, id: &str) -> Option<UserEntity> {
        let conn = self.conn.lock().ok()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, title, content, author, confidence, evidence_count, last_validated, tags, relations, created_at, updated_at FROM user_entities WHERE id=?1",
            )
            .ok()?;
        stmt.query_row(params![id], row_to_user_entity).ok()
    }

    fn all_user_entity_ids(&self) -> Vec<String> {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };
        let mut stmt = match conn.prepare("SELECT id FROM user_entities") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let rows = match stmt.query_map([], |row| row.get::<_, String>(0)) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };
        rows.filter_map(|r| r.ok()).collect()
    }

    fn all_user_entities(&self) -> Vec<UserEntity> {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };
        let mut stmt = match conn.prepare(
            "SELECT id, title, content, author, confidence, evidence_count,
                    last_validated, tags, relations, created_at, updated_at
             FROM user_entities",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let rows = stmt.query_map([], row_to_user_entity);
        match rows {
            Ok(r) => r.filter_map(|x| x.ok()).collect(),
            Err(_) => Vec::new(),
        }
    }

    fn get_user_neighbors(&self, entity_id: &str, relation_type: Option<&str>) -> Vec<String> {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };
        let mut neighbors = Vec::new();

        // Outgoing
        query_neighbors(
            &conn,
            "SELECT to_id FROM user_relations WHERE from_id=?1",
            entity_id,
            relation_type,
            &mut neighbors,
        );
        // Incoming
        query_neighbors(
            &conn,
            "SELECT from_id FROM user_relations WHERE to_id=?1",
            entity_id,
            relation_type,
            &mut neighbors,
        );

        neighbors.sort();
        neighbors.dedup();
        neighbors
    }

    fn get_user_all_edges(&self, entity_id: &str) -> Vec<GraphEdge> {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };
        let mut edges = Vec::new();

        // Outgoing
        let mut stmt = match conn
            .prepare("SELECT from_id, relation_type, to_id FROM user_relations WHERE from_id=?1")
        {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let rows = stmt.query_map(params![entity_id], |row| {
            Ok(GraphEdge {
                from_id: row.get(0)?,
                relation_type: row.get(1)?,
                to_id: row.get(2)?,
            })
        });
        if let Ok(rows) = rows {
            for r in rows.flatten() {
                edges.push(r);
            }
        }

        // Incoming
        let mut stmt = match conn
            .prepare("SELECT from_id, relation_type, to_id FROM user_relations WHERE to_id=?1")
        {
            Ok(s) => s,
            Err(_) => return edges,
        };
        let rows = stmt.query_map(params![entity_id], |row| {
            Ok(GraphEdge {
                from_id: row.get(0)?,
                relation_type: row.get(1)?,
                to_id: row.get(2)?,
            })
        });
        if let Ok(rows) = rows {
            for r in rows.flatten() {
                edges.push(r);
            }
        }

        edges
    }

    fn search_user_entities(&self, query: &str, limit: usize) -> Vec<UserEntity> {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };
        let mut stmt = match conn.prepare(
            "SELECT u.id, u.title, u.content, u.author, u.confidence, u.evidence_count,
                    u.last_validated, u.tags, u.relations, u.created_at, u.updated_at
             FROM user_entities_fts f
             JOIN user_entities u ON u.rowid = f.rowid
             WHERE user_entities_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let rows = stmt.query_map(params![query, limit as i64], row_to_user_entity);
        match rows {
            Ok(r) => r.filter_map(|x| x.ok()).collect(),
            Err(_) => Vec::new(),
        }
    }

    fn compute_correlations(&self, insight_id: &str) -> Vec<CorrelationScore> {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };
        // Get the target's neighbor targets as a flat set
        let target_targets: std::collections::HashSet<String> = {
            let mut stmt = match conn.prepare("SELECT to_id FROM user_relations WHERE from_id=?1") {
                Ok(s) => s,
                Err(_) => return Vec::new(),
            };
            let rows = stmt.query_map(params![insight_id], |row| row.get::<_, String>(0));
            match rows {
                Ok(r) => r.filter_map(|v| v.ok()).collect(),
                Err(_) => return Vec::new(),
            }
        };

        // Single query: get all other user entities with their neighbor targets
        // using a self-JOIN to compute Jaccard in SQL-adjacent logic
        let mut stmt = match conn.prepare(
            "SELECT r1.from_id, r1.to_id
             FROM user_relations r1
             WHERE r1.from_id != ?1",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let rows = stmt.query_map(params![insight_id], |row| {
            let from: String = row.get(0)?;
            let to: String = row.get(1)?;
            Ok((from, to))
        });

        // Group by entity: entity_id -> set of targets
        let mut entity_targets: HashMap<String, std::collections::HashSet<String>> = HashMap::new();
        if let Ok(rows) = rows {
            for r in rows.flatten() {
                entity_targets.entry(r.0).or_default().insert(r.1);
            }
        }

        let mut scores = Vec::new();
        for (other_id, other_targets) in &entity_targets {
            let intersection = target_targets.intersection(other_targets).count();
            let union = target_targets.union(other_targets).count();
            let graph_prox = if union == 0 {
                0.0
            } else {
                intersection as f64 / union as f64
            };

            let combined = 0.4 * graph_prox + 0.2 * 0.5;
            if combined > 0.05 {
                scores.push(CorrelationScore {
                    insight_id: other_id.clone(),
                    semantic: 0.0,
                    graph_proximity: graph_prox,
                    temporal: 0.5,
                    combined,
                });
            }
        }

        scores.sort_by(|a, b| {
            b.combined
                .partial_cmp(&a.combined)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scores
    }

    fn store_embedding(&self, entity_id: &str, embedding: &[f32]) -> Result<(), String> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("mutex poisoned: {e}"))?;
        let bytes: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();
        conn.execute(
            "INSERT OR REPLACE INTO user_embeddings (entity_id, embedding) VALUES (?1, ?2)",
            params![entity_id, bytes],
        )
        .map_err(|e| format!("store embedding: {e}"))?;
        Ok(())
    }

    fn get_embedding(&self, entity_id: &str) -> Option<Vec<f32>> {
        let conn = self.conn.lock().ok()?;
        let blob: Vec<u8> = conn
            .query_row(
                "SELECT embedding FROM user_embeddings WHERE entity_id=?1",
                params![entity_id],
                |row| row.get(0),
            )
            .ok()?;
        blob.chunks_exact(4)
            .map(|chunk| {
                let bytes: [u8; 4] = [chunk[0], chunk[1], chunk[2], chunk[3]];
                f32::from_le_bytes(bytes)
            })
            .collect::<Vec<_>>()
            .into()
    }

    fn user_entity_count(&self) -> usize {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return 0,
        };
        conn.query_row("SELECT COUNT(*) FROM user_entities", [], |row| {
            row.get::<_, i64>(0)
        })
        .ok()
        .and_then(|c| usize::try_from(c).ok())
        .unwrap_or(0)
    }

    /// Override: use atomic SQLite sequence counter instead of scanning.
    fn next_insight_id(&self) -> Result<String, String> {
        self.next_insight_id_atomic()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entity(id: &str, title: &str) -> UserEntity {
        UserEntity {
            id: id.to_owned(),
            title: title.to_owned(),
            content: format!("Content for {title}"),
            author: "test".to_owned(),
            confidence: 0.5,
            evidence_count: 0,
            last_validated: String::new(),
            tags: vec!["test".to_owned()],
            relations: HashMap::new(),
            created_at: "2026-01-01T00:00:00Z".to_owned(),
            updated_at: "2026-01-01T00:00:00Z".to_owned(),
        }
    }

    #[test]
    fn open_in_memory_initializes_schema() {
        let store = UserGraphStore::open_in_memory().unwrap();
        assert_eq!(store.user_entity_count(), 0);
    }

    #[test]
    fn add_and_get_entity() {
        let store = UserGraphStore::open_in_memory().unwrap();
        let entity = make_entity("TK-001", "Test Insight");
        store.add_entity(entity).unwrap();
        assert_eq!(store.user_entity_count(), 1);

        let retrieved = store.get_user_entity("TK-001").unwrap();
        assert_eq!(retrieved.title, "Test Insight");
        assert_eq!(retrieved.author, "test");
    }

    #[test]
    fn add_duplicate_entity_fails() {
        let store = UserGraphStore::open_in_memory().unwrap();
        store.add_entity(make_entity("TK-001", "First")).unwrap();
        let result = store.add_entity(make_entity("TK-001", "Second"));
        assert!(result.is_err());
    }

    #[test]
    fn update_entity() {
        let store = UserGraphStore::open_in_memory().unwrap();
        store.add_entity(make_entity("TK-001", "Original")).unwrap();
        let mut updated = make_entity("TK-001", "Updated");
        updated.confidence = 0.9;
        store.update_entity("TK-001", updated).unwrap();

        let retrieved = store.get_user_entity("TK-001").unwrap();
        assert_eq!(retrieved.title, "Updated");
        assert!((retrieved.confidence - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn remove_entity_cascades() {
        let store = UserGraphStore::open_in_memory().unwrap();
        store.add_entity(make_entity("TK-001", "A")).unwrap();
        store.add_entity(make_entity("TK-002", "B")).unwrap();
        store
            .add_relation("TK-001", "derives_from", "TK-002")
            .unwrap();

        store.remove_entity("TK-001").unwrap();
        assert_eq!(store.user_entity_count(), 1);
        assert!(store.get_user_entity("TK-001").is_none());
        // TK-002 still exists but relation to TK-001 is gone
        let neighbors = store.get_user_neighbors("TK-002", None);
        assert!(neighbors.is_empty());
    }

    #[test]
    fn add_and_remove_relation() {
        let store = UserGraphStore::open_in_memory().unwrap();
        store.add_entity(make_entity("TK-001", "A")).unwrap();
        store.add_entity(make_entity("TK-002", "B")).unwrap();
        store
            .add_relation("TK-001", "derives_from", "TK-002")
            .unwrap();

        let neighbors = store.get_user_neighbors("TK-001", Some("derives_from"));
        assert_eq!(neighbors, vec!["TK-002"]);

        store
            .remove_relation("TK-001", "derives_from", "TK-002")
            .unwrap();
        let neighbors = store.get_user_neighbors("TK-001", Some("derives_from"));
        assert!(neighbors.is_empty());
    }

    #[test]
    fn search_by_keyword() {
        let store = UserGraphStore::open_in_memory().unwrap();
        let mut entity = make_entity("TK-001", "Strategy Pattern Decision");
        entity.content = "We decided to use Strategy for payment processing".to_owned();
        entity.tags = vec!["decision".to_owned(), "payment".to_owned()];
        store.add_entity(entity).unwrap();

        let results = store.search_user_entities("Strategy payment", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "TK-001");
    }

    #[test]
    fn store_and_retrieve_embedding() {
        let store = UserGraphStore::open_in_memory().unwrap();
        store.add_entity(make_entity("TK-001", "Embedded")).unwrap();
        let embedding = vec![0.1, 0.2, 0.3, 0.4];
        store.store_embedding("TK-001", &embedding).unwrap();

        let retrieved = store.get_embedding("TK-001").unwrap();
        assert_eq!(retrieved.len(), 4);
        assert!((retrieved[0] - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn compute_correlations_shared_neighbors() {
        let store = UserGraphStore::open_in_memory().unwrap();
        store.add_entity(make_entity("TK-001", "A")).unwrap();
        store.add_entity(make_entity("TK-002", "B")).unwrap();
        // Both derive from same canonical entity
        store
            .add_relation("TK-001", "derives_from", "DP-005")
            .unwrap();
        store
            .add_relation("TK-002", "derives_from", "DP-005")
            .unwrap();

        let correlations = store.compute_correlations("TK-001");
        assert_eq!(correlations.len(), 1);
        assert_eq!(correlations[0].insight_id, "TK-002");
        assert!(correlations[0].graph_proximity > 0.9);
    }

    #[test]
    fn get_all_edges() {
        let store = UserGraphStore::open_in_memory().unwrap();
        store.add_entity(make_entity("TK-001", "A")).unwrap();
        store.add_entity(make_entity("TK-002", "B")).unwrap();
        store
            .add_relation("TK-001", "derives_from", "DP-005")
            .unwrap();
        store
            .add_relation("TK-001", "relates_to", "TK-002")
            .unwrap();

        let edges = store.get_user_all_edges("TK-001");
        assert_eq!(edges.len(), 2);
    }
}
