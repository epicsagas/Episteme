use rusqlite::{Connection, params};

use crate::adapters::error::{InfraError, Result};

// ---------------------------------------------------------------------------
// Schema
// ---------------------------------------------------------------------------

/// Current schema version (bumped when DDL changes).
const SCHEMA_VERSION: u32 = 1;

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
