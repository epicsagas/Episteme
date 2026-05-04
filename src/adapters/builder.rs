use std::collections::HashMap;
use std::path::Path;

use rusqlite::Connection;

use crate::adapters::error::{InfraError, Result};
use crate::adapters::json_loader::load_graph;
use crate::adapters::sqlite_db::{self, Chunk};
use crate::domain::types::Entity;
use crate::ports::embeddings::EmbeddingProvider;
use crate::adapters::chunker;

// ---------------------------------------------------------------------------
// Build statistics
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct BuildStats {
    pub files_scanned: usize,
    pub chunks_created: usize,
    pub embeddings_generated: usize,
    pub skipped_no_file: usize,
}

// ---------------------------------------------------------------------------
// Build pipeline
// ---------------------------------------------------------------------------

/// Run the full RAG build pipeline:
/// 1. Open / init the SQLite database at `db_path`.
/// 2. Load `file_to_entity.json` from `data_dir`.
/// 3. For each file, chunk the markdown and insert into the DB.
/// 4. Generate embeddings for any chunks that do not yet have one.
pub fn build(
    db_path: &Path,
    data_dir: &Path,
    raw_dir: &Path,
    provider: &dyn EmbeddingProvider,
    batch_size: usize,
) -> Result<BuildStats> {
    // Open database and initialise schema.
    let conn = Connection::open(db_path).map_err(|e| InfraError::Database(e.to_string()))?;
    sqlite_db::init_database(&conn)?;

    // Load file_to_entity mapping.
    let f2e_path = data_dir.join("file_to_entity.json");
    let f2e_raw = std::fs::read_to_string(&f2e_path).map_err(InfraError::Io)?;
    let file_to_entity: HashMap<String, String> = serde_json::from_str(&f2e_raw).map_err(InfraError::Json)?;

    // Load the knowledge graph for entity metadata.
    let kg = load_graph(data_dir)?;

    let mut stats = BuildStats {
        files_scanned: 0,
        chunks_created: 0,
        embeddings_generated: 0,
        skipped_no_file: 0,
    };

    let mut all_chunks: Vec<Chunk> = Vec::new();

    for (file_path_str, entity_id) in &file_to_entity {
        let file_path = raw_dir.join(file_path_str);

        if !file_path.exists() {
            stats.skipped_no_file += 1;
            continue;
        }

        // Skip README files and Korean translations (mirrors Python).
        let fname = file_path.file_name().unwrap_or_default().to_string_lossy();
        let path_str = file_path_str.as_str();
        if fname.contains("README") || path_str.contains("/ko/") {
            continue;
        }

        let entity = match kg.get_entity(entity_id) {
            Some(e) => e.clone(),
            None => Entity {
                id: entity_id.clone(),
                r#type: "unknown".to_owned(),
                title: String::new(),
                name: String::new(),
                category: String::new(),
                tags: Vec::new(),
                relations: HashMap::new(),
                context: HashMap::new(),
                file_path: String::new(),
                source: serde_json::Value::Null,
            },
        };

        let chunks = chunker::chunk_markdown(&file_path, entity_id, &entity)?;
        stats.files_scanned += 1;
        stats.chunks_created += chunks.len();
        all_chunks.extend(chunks);
    }

    // Insert all chunks.
    sqlite_db::insert_chunks(&conn, &all_chunks)?;

    // Generate embeddings for chunks that do not yet have one.
    stats.embeddings_generated = generate_embeddings(&conn, provider, batch_size)?;

    Ok(stats)
}

// ---------------------------------------------------------------------------
// Embedding generation
// ---------------------------------------------------------------------------

/// Generate embeddings for chunks that do not yet have one.
fn generate_embeddings(
    conn: &Connection,
    provider: &dyn EmbeddingProvider,
    batch_size: usize,
) -> Result<usize> {
    // Find chunks without embeddings.
    let mut stmt = conn
        .prepare(
            "SELECT id, text FROM chunks
             WHERE id NOT IN (SELECT chunk_id FROM embeddings)",
        )
        .map_err(|e| InfraError::Database(e.to_string()))?;

    let rows: Vec<(String, String)> = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| InfraError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    if rows.is_empty() {
        return Ok(0);
    }

    let texts: Vec<&str> = rows.iter().map(|(_, t)| t.as_str()).collect();
    let embeddings = provider
        .embed_batch(&texts, batch_size.max(1))
        .map_err(InfraError::Embedding)?;

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| InfraError::Database(e.to_string()))?;

    for ((chunk_id, _), embedding) in rows.iter().zip(embeddings.iter()) {
        let blob: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();
        tx.execute(
            "INSERT OR REPLACE INTO embeddings (chunk_id, embedding) VALUES (?1, ?2)",
            rusqlite::params![chunk_id, blob],
        )
        .map_err(|e| InfraError::Database(e.to_string()))?;
    }

    tx.commit().map_err(|e| InfraError::Database(e.to_string()))?;

    Ok(rows.len())
}
