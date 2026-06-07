use std::collections::HashMap;
use std::str::FromStr;

use rusqlite::{Connection, params};

use crate::adapters::error::{InfraError, Result};
use crate::domain::types::{Entity, EntityType};

// ---------------------------------------------------------------------------
// Schema
// ---------------------------------------------------------------------------

/// Current schema version (bumped when DDL changes).
const SCHEMA_VERSION: u32 = 2;

/// Full DDL for the RAG database.
///
/// Includes `_meta` table required by [`llm_kernel::store::init_schema`].
const SCHEMA_DDL: &str = "
    CREATE TABLE IF NOT EXISTS _meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
    INSERT OR IGNORE INTO _meta (key, value) VALUES ('schema_version', '0');

    CREATE TABLE IF NOT EXISTS chunks (
        id TEXT PRIMARY KEY,
        text TEXT NOT NULL,
        entity_id TEXT NOT NULL,
        entity_type TEXT NOT NULL,
        title TEXT,
        section TEXT,
        chunk_index INTEGER,
        metadata TEXT,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE IF NOT EXISTS embeddings (
        chunk_id TEXT PRIMARY KEY,
        embedding BLOB NOT NULL,
        FOREIGN KEY (chunk_id) REFERENCES chunks(id)
    );

    CREATE INDEX IF NOT EXISTS idx_entity_id ON chunks(entity_id);
    CREATE INDEX IF NOT EXISTS idx_entity_type ON chunks(entity_type);

    -- Knowledge graph tables (schema version 2)
    CREATE TABLE IF NOT EXISTS entities (
        name TEXT PRIMARY KEY,
        entity_type TEXT NOT NULL,
        category TEXT,
        description TEXT,
        tags TEXT,
        attributes TEXT,
        file_path TEXT
    );

    CREATE TABLE IF NOT EXISTS relations (
        source TEXT NOT NULL,
        target TEXT NOT NULL,
        relation_type TEXT NOT NULL,
        PRIMARY KEY (source, target, relation_type),
        FOREIGN KEY (source) REFERENCES entities(name),
        FOREIGN KEY (target) REFERENCES entities(name)
    );

    CREATE INDEX IF NOT EXISTS idx_relations_target ON relations(target);
";

// ---------------------------------------------------------------------------
// Database initialization
// ---------------------------------------------------------------------------

/// Initialize the RAG database at `path` using llm-kernel's `init_schema`.
///
/// Creates parent directories, applies PRAGMAs (WAL, foreign keys, busy timeout),
/// runs the DDL, and validates schema versioning.
pub fn open_database(path: &std::path::Path) -> Result<Connection> {
    llm_kernel::store::init_schema(path, SCHEMA_DDL, SCHEMA_VERSION)
        .map_err(|e| InfraError::Database(e.to_string()))
}

/// Initialize an in-memory database for testing.
pub fn init_in_memory() -> Result<Connection> {
    llm_kernel::store::init_in_memory(SCHEMA_DDL).map_err(|e| InfraError::Database(e.to_string()))
}

// ---------------------------------------------------------------------------
// _meta key-value helpers
// ---------------------------------------------------------------------------

/// Read a value from the `_meta` table. Returns `None` if the key does not exist.
pub fn get_meta(conn: &Connection, key: &str) -> Result<Option<String>> {
    match conn.query_row(
        "SELECT value FROM _meta WHERE key = ?1",
        params![key],
        |row| row.get::<_, String>(0),
    ) {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(InfraError::Database(e.to_string())),
    }
}

/// Write (upsert) a value into the `_meta` table.
pub fn set_meta(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO _meta (key, value) VALUES (?1, ?2)",
        params![key, value],
    )
    .map_err(|e| InfraError::Database(e.to_string()))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Chunk type
// ---------------------------------------------------------------------------

/// A text chunk with associated metadata, ready for insertion into SQLite.
#[derive(Debug, Clone)]
pub struct Chunk {
    pub id: String,
    pub text: String,
    pub entity_id: String,
    pub entity_type: String,
    pub title: String,
    pub section: String,
    pub chunk_index: i64,
    pub metadata: String, // JSON-encoded
}

// ---------------------------------------------------------------------------
// Insert
// ---------------------------------------------------------------------------

/// Insert (or replace) a batch of chunks.
pub fn insert_chunks(conn: &Connection, chunks: &[Chunk]) -> Result<()> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| InfraError::Database(e.to_string()))?;

    for chunk in chunks {
        tx.execute(
            "INSERT OR REPLACE INTO chunks
             (id, text, entity_id, entity_type, title, section, chunk_index, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                chunk.id,
                chunk.text,
                chunk.entity_id,
                chunk.entity_type,
                chunk.title,
                chunk.section,
                chunk.chunk_index,
                chunk.metadata,
            ],
        )
        .map_err(|e| InfraError::Database(e.to_string()))?;
    }

    tx.commit()
        .map_err(|e| InfraError::Database(e.to_string()))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Queries
// ---------------------------------------------------------------------------

/// Row returned by `get_all_embeddings`.
#[derive(Debug)]
pub struct EmbeddingRow {
    pub chunk_id: String,
    pub text: String,
    pub entity_id: String,
    pub entity_type: String,
    pub title: String,
    pub section: String,
    pub metadata: String,
    pub embedding: Vec<u8>,
}

/// Fetch every (chunk, embedding) pair, optionally filtered by
/// `entity_type` and/or `entity_id`.
pub fn get_all_embeddings(
    conn: &Connection,
    entity_type: Option<&str>,
    entity_id: Option<&str>,
) -> Result<Vec<EmbeddingRow>> {
    let mut sql = String::from(
        "SELECT c.id, c.text, c.entity_id, c.entity_type, c.title, c.section, c.metadata, e.embedding
         FROM chunks c
         JOIN embeddings e ON c.id = e.chunk_id
         WHERE 1=1",
    );
    let mut p: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(et) = entity_type {
        sql.push_str(" AND c.entity_type = ?");
        p.push(Box::new(et.to_owned()));
    }
    if let Some(eid) = entity_id {
        sql.push_str(" AND c.entity_id = ?");
        p.push(Box::new(eid.to_owned()));
    }

    let params_refs: Vec<&dyn rusqlite::types::ToSql> = p.iter().map(|x| x.as_ref()).collect();

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| InfraError::Database(e.to_string()))?;

    let rows = stmt
        .query_map(params_refs.as_slice(), |row| {
            Ok(EmbeddingRow {
                chunk_id: row.get(0)?,
                text: row.get(1)?,
                entity_id: row.get(2)?,
                entity_type: row.get(3)?,
                title: row.get(4)?,
                section: row.get(5)?,
                metadata: row.get(6)?,
                embedding: row.get(7)?,
            })
        })
        .map_err(|e| InfraError::Database(e.to_string()))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| InfraError::Database(e.to_string()))?);
    }
    Ok(results)
}

/// Return the total number of chunks in the database.
pub fn get_chunk_count(conn: &Connection) -> Result<usize> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM chunks", [], |row| row.get(0))
        .map_err(|e| InfraError::Database(e.to_string()))?;
    usize::try_from(count).map_err(|_| InfraError::Database("chunk count overflow".to_owned()))
}

/// Return the total number of embeddings in the database.
pub fn get_embedding_count(conn: &Connection) -> Result<usize> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM embeddings", [], |row| row.get(0))
        .map_err(|e| InfraError::Database(e.to_string()))?;
    usize::try_from(count).map_err(|_| InfraError::Database("embedding count overflow".to_owned()))
}

// ---------------------------------------------------------------------------
// Graph tables: entities + relations
// ---------------------------------------------------------------------------

/// Upsert entities and their relations into the DB.
///
/// Uses INSERT OR REPLACE so only the supplied entities are touched;
/// existing entities not in the input are left unchanged. Relations for
/// each upserted entity are replaced (delete-then-insert) to stay in sync.
pub fn insert_graph(conn: &Connection, entities: &HashMap<String, Entity>) -> Result<()> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| InfraError::Database(e.to_string()))?;

    // Pass 1: upsert all entities first so FK targets exist.
    for (id, entity) in entities {
        // Normalize entity_type to the canonical lowercase form via the domain enum.
        let entity_type_str = EntityType::from_str(&entity.r#type)
            .map(|et| et.to_string())
            .unwrap_or_else(|_| entity.r#type.clone());

        let tags_json = serde_json::to_string(&entity.tags).unwrap_or_else(|_| "[]".to_owned());
        // Serialize only fields NOT stored in dedicated columns.
        let attrs = serde_json::json!({
            "title": entity.title,
            "name": entity.name,
            "context": entity.context,
            "source": entity.source,
        });
        let attrs_json = serde_json::to_string(&attrs).unwrap_or_else(|_| "{}".to_owned());

        tx.execute(
            "INSERT OR REPLACE INTO entities (name, entity_type, category, description, tags, attributes, file_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                id,
                entity_type_str,
                entity.category,
                entity.description,
                tags_json,
                attrs_json,
                entity.file_path,
            ],
        )
        .map_err(|e| InfraError::Database(e.to_string()))?;
    }

    // Pass 2: replace relations per entity.
    for (id, entity) in entities {
        tx.execute("DELETE FROM relations WHERE source = ?1", params![id])
            .map_err(|e| InfraError::Database(e.to_string()))?;

        for (rel_type, targets) in &entity.relations {
            for target in targets {
                tx.execute(
                    "INSERT OR IGNORE INTO relations (source, target, relation_type) VALUES (?1, ?2, ?3)",
                    params![id, target, rel_type],
                )
                .map_err(|e| InfraError::Database(e.to_string()))?;
            }
        }
    }

    tx.commit()
        .map_err(|e| InfraError::Database(e.to_string()))?;
    Ok(())
}

/// Load all entities and relations from the DB into a knowledge graph.
///
/// Returns `None` if the entities table is empty (no graph data in DB).
pub fn load_graph_from_db(conn: &Connection) -> Result<Option<HashMap<String, Entity>>> {
    let entity_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM entities", [], |row| row.get(0))
        .map_err(|e| InfraError::Database(e.to_string()))?;

    if entity_count == 0 {
        return Ok(None);
    }

    let mut entities = HashMap::new();

    // Load entities
    let mut stmt = conn
        .prepare("SELECT name, entity_type, category, description, tags, attributes, file_path FROM entities")
        .map_err(|e| InfraError::Database(e.to_string()))?;

    let entity_rows = stmt
        .query_map([], |row| {
            let name: String = row.get(0)?;
            let entity_type: String = row.get(1)?;
            let category: Option<String> = row.get(2)?;
            let description: Option<String> = row.get(3)?;
            let tags_json: Option<String> = row.get(4)?;
            let attrs_json: Option<String> = row.get(5)?;
            let file_path: Option<String> = row.get(6)?;

            Ok((
                name,
                entity_type,
                category,
                description,
                tags_json,
                attrs_json,
                file_path,
            ))
        })
        .map_err(|e| InfraError::Database(e.to_string()))?;

    for row in entity_rows {
        let (id, entity_type, category, description, tags_json, attrs_json, file_path) =
            row.map_err(|e| InfraError::Database(e.to_string()))?;

        // Normalize and validate entity_type against the domain enum.
        let normalized_type = match EntityType::from_str(&entity_type) {
            Ok(et) => et.to_string(),
            Err(_) => {
                tracing::warn!(id = %id, entity_type = %entity_type, "skipping entity with invalid type");
                continue;
            }
        };

        let tags: Vec<String> = tags_json
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();
        let attrs: serde_json::Value = attrs_json
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or(serde_json::Value::Object(Default::default()));

        let entity = Entity {
            id: id.clone(),
            r#type: normalized_type,
            title: attrs
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_owned(),
            description: description.unwrap_or_default(),
            name: attrs
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_owned(),
            category: category.unwrap_or_default(),
            tags,
            relations: HashMap::new(),
            context: attrs
                .get("context")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default(),
            file_path: file_path.unwrap_or_default(),
            source: attrs
                .get("source")
                .cloned()
                .unwrap_or(serde_json::Value::Null),
        };
        entities.insert(id, entity);
    }

    // Load relations, skipping orphans whose source or target entity was discarded.
    let mut rel_stmt = conn
        .prepare("SELECT source, target, relation_type FROM relations")
        .map_err(|e| InfraError::Database(e.to_string()))?;

    let rel_rows = rel_stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| InfraError::Database(e.to_string()))?;

    for row in rel_rows {
        let (source, target, rel_type) = row.map_err(|e| InfraError::Database(e.to_string()))?;
        // Skip if source entity was discarded or target entity is missing.
        if !entities.contains_key(&target) {
            tracing::debug!(source = %source, target = %target, "skipping orphan relation");
            continue;
        }
        if let Some(entity) = entities.get_mut(&source) {
            entity.relations.entry(rel_type).or_default().push(target);
        }
    }

    Ok(Some(entities))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a minimal entity with the given id and type.
    fn make_entity(id: &str, entity_type: &str) -> Entity {
        Entity {
            id: id.to_owned(),
            r#type: entity_type.to_owned(),
            ..Default::default()
        }
    }

    /// Helper: build a fully-populated entity for exhaustive roundtrip testing.
    fn make_full_entity(id: &str) -> Entity {
        let mut relations = HashMap::new();
        relations.insert("solves".to_owned(), vec!["SMELL-01".to_owned()]);
        let mut context = HashMap::new();
        context.insert("when".to_owned(), vec!["long methods".to_owned()]);
        Entity {
            id: id.to_owned(),
            r#type: "refactoring".to_owned(),
            title: "Extract Method".to_owned(),
            description: "Decompose long methods into smaller ones.".to_owned(),
            name: "Extract Method".to_owned(),
            category: "composition".to_owned(),
            tags: vec!["methods".to_owned(), "decomposition".to_owned()],
            relations,
            context,
            file_path: "refactorings/extract-method.md".to_owned(),
            source: serde_json::json!({"url": "https://example.com"}),
        }
    }

    /// Helper: compare two entities for semantic equality, ignoring relations
    /// ordering (since DB roundtrip may reorder).
    fn assert_entity_eq(a: &Entity, b: &Entity) {
        assert_eq!(a.id, b.id);
        assert_eq!(a.r#type, b.r#type);
        assert_eq!(a.title, b.title);
        assert_eq!(a.description, b.description);
        assert_eq!(a.name, b.name);
        assert_eq!(a.category, b.category);
        let mut a_tags = a.tags.clone();
        let mut b_tags = b.tags.clone();
        a_tags.sort();
        b_tags.sort();
        assert_eq!(a_tags, b_tags);
        assert_eq!(a.file_path, b.file_path);
        assert_eq!(a.source, b.source);
        assert_eq!(a.context, b.context);
        // Relations: compare as sorted multisets per key.
        let a_keys: std::collections::HashSet<_> = a.relations.keys().collect();
        let b_keys: std::collections::HashSet<_> = b.relations.keys().collect();
        assert_eq!(a_keys, b_keys, "relation keys mismatch");
        for key in &a_keys {
            let mut av = a.relations[*key].clone();
            let mut bv = b.relations[*key].clone();
            av.sort();
            bv.sort();
            assert_eq!(av, bv, "relations[{key}] mismatch");
        }
    }

    #[test]
    fn insert_and_load_empty_graph() {
        let conn = init_in_memory().expect("in-memory DB");
        let empty: HashMap<String, Entity> = HashMap::new();
        insert_graph(&conn, &empty).expect("insert empty");
        let loaded = load_graph_from_db(&conn).expect("load empty");
        assert!(loaded.is_none(), "empty DB should return None");
    }

    #[test]
    fn insert_and_load_single_entity() {
        let conn = init_in_memory().expect("in-memory DB");
        let e = make_entity("SMELL-01", "smell");
        let mut map = HashMap::new();
        map.insert("SMELL-01".to_owned(), e.clone());
        insert_graph(&conn, &map).expect("insert");

        let loaded = load_graph_from_db(&conn)
            .expect("load")
            .expect("should have data");
        assert_eq!(loaded.len(), 1);
        assert_entity_eq(&e, &loaded["SMELL-01"]);
    }

    #[test]
    fn insert_and_load_full_entity_roundtrip() {
        let conn = init_in_memory().expect("in-memory DB");
        let e = make_full_entity("RF-001");
        let target = make_entity("SMELL-01", "smell");
        let mut map = HashMap::new();
        map.insert("RF-001".to_owned(), e.clone());
        map.insert("SMELL-01".to_owned(), target);
        insert_graph(&conn, &map).expect("insert");

        let loaded = load_graph_from_db(&conn)
            .expect("load")
            .expect("should have data");
        assert_eq!(loaded.len(), 2);
        assert_entity_eq(&e, &loaded["RF-001"]);
    }

    #[test]
    fn insert_and_load_entities_with_relations() {
        let conn = init_in_memory().expect("in-memory DB");

        let smell = make_entity("SMELL-01", "smell");
        let mut rf = make_entity("RF-001", "refactoring");
        rf.relations
            .insert("solves".to_owned(), vec!["SMELL-01".to_owned()]);

        let mut map = HashMap::new();
        map.insert("SMELL-01".to_owned(), smell.clone());
        map.insert("RF-001".to_owned(), rf.clone());
        insert_graph(&conn, &map).expect("insert");

        let loaded = load_graph_from_db(&conn)
            .expect("load")
            .expect("should have data");
        assert_eq!(loaded.len(), 2);
        assert_entity_eq(&smell, &loaded["SMELL-01"]);
        assert_entity_eq(&rf, &loaded["RF-001"]);
    }

    #[test]
    fn insert_graph_is_idempotent() {
        let conn = init_in_memory().expect("in-memory DB");
        let e = make_full_entity("RF-001");
        let target = make_entity("SMELL-01", "smell");
        let mut map = HashMap::new();
        map.insert("RF-001".to_owned(), e.clone());
        map.insert("SMELL-01".to_owned(), target);

        insert_graph(&conn, &map).expect("insert 1");
        insert_graph(&conn, &map).expect("insert 2");

        let loaded = load_graph_from_db(&conn)
            .expect("load")
            .expect("should have data");
        assert_eq!(loaded.len(), 2);
        assert_entity_eq(&e, &loaded["RF-001"]);
    }

    #[test]
    fn load_skips_invalid_entity_type() {
        let conn = init_in_memory().expect("in-memory DB");

        // Insert a valid entity and a bogus one directly via SQL.
        conn.execute(
            "INSERT INTO entities (name, entity_type, category, description, tags, attributes, file_path)
             VALUES ('SMELL-01', 'smell', '', '', '[]', '{}', '')",
            [],
        )
        .expect("insert valid");
        conn.execute(
            "INSERT INTO entities (name, entity_type, category, description, tags, attributes, file_path)
             VALUES ('BOGUS-01', 'not_a_real_type', '', '', '[]', '{}', '')",
            [],
        )
        .expect("insert bogus");

        let loaded = load_graph_from_db(&conn)
            .expect("load")
            .expect("should have data");
        assert_eq!(loaded.len(), 1, "only the valid entity should be loaded");
        assert!(loaded.contains_key("SMELL-01"));
    }

    #[test]
    fn load_handles_null_columns() {
        let conn = init_in_memory().expect("in-memory DB");

        // Insert an entity with NULLs in nullable columns.
        conn.execute(
            "INSERT INTO entities (name, entity_type, category, description, tags, attributes, file_path)
             VALUES ('SMELL-01', 'smell', NULL, NULL, NULL, NULL, NULL)",
            [],
        )
        .expect("insert with nulls");

        let loaded = load_graph_from_db(&conn)
            .expect("load")
            .expect("should have data");
        assert_eq!(loaded.len(), 1);
        let e = &loaded["SMELL-01"];
        assert_eq!(e.category, "");
        assert_eq!(e.description, "");
        assert_eq!(e.file_path, "");
        assert!(e.tags.is_empty());
    }

    #[test]
    fn load_skips_orphan_relations() {
        let conn = init_in_memory().expect("in-memory DB");

        // Insert a valid entity and a bogus one, plus a relation from valid → bogus.
        conn.execute(
            "INSERT INTO entities (name, entity_type, category, description, tags, attributes, file_path)
             VALUES ('RF-001', 'refactoring', '', '', '[]', '{}', '')",
            [],
        )
        .expect("insert valid");
        conn.execute(
            "INSERT INTO entities (name, entity_type, category, description, tags, attributes, file_path)
             VALUES ('BOGUS-01', 'not_a_real_type', '', '', '[]', '{}', '')",
            [],
        )
        .expect("insert bogus");
        conn.execute(
            "INSERT INTO relations (source, target, relation_type) VALUES ('RF-001', 'BOGUS-01', 'solves')",
            [],
        )
        .expect("insert relation to bogus");

        let loaded = load_graph_from_db(&conn)
            .expect("load")
            .expect("should have data");
        assert_eq!(loaded.len(), 1, "only the valid entity should be loaded");
        let rf = &loaded["RF-001"];
        assert!(
            rf.relations.is_empty(),
            "orphan relation to discarded entity should be skipped"
        );
    }

    #[test]
    fn insert_normalizes_entity_type() {
        let conn = init_in_memory().expect("in-memory DB");

        // Entity with non-canonical casing in r#type.
        let mut e = Entity::default();
        e.id = "SMELL-01".to_owned();
        e.r#type = "Smell".to_owned(); // non-canonical casing
        let mut map = HashMap::new();
        map.insert("SMELL-01".to_owned(), e);

        insert_graph(&conn, &map).expect("insert");

        let loaded = load_graph_from_db(&conn)
            .expect("load")
            .expect("should have data");
        assert_eq!(
            loaded["SMELL-01"].r#type, "smell",
            "entity_type should be normalized"
        );
    }
}
