use std::collections::HashMap;
use std::path::Path;

use crate::adapters::error::{InfraError, Result};
use crate::adapters::paths;
use crate::adapters::sqlite_db;
use crate::domain::graph::KnowledgeGraph;
use crate::domain::types::Entity;

const ENTITY_PREFIXES: &[&str] = &["DP-", "RF-", "LAW-", "SMELL-"];

/// Load the knowledge graph, trying the DB first.
///
/// 1. If `episteme.db` exists and has graph data in `entities` table → load from DB.
/// 2. Otherwise, fall back to `relations.json` + `raw/` enrichment.
pub fn load_graph(data_dir: &Path) -> Result<KnowledgeGraph> {
    if let Some(entities) = try_load_graph_from_db()? {
        return Ok(KnowledgeGraph::from_entities(entities));
    }
    load_graph_from_json(data_dir)
}

/// Try loading the knowledge graph from the SQLite DB.
///
/// Returns `Ok(Some(..))` on success, `Ok(None)` if the DB has no graph data,
/// and falls through silently on any I/O or query error.
fn try_load_graph_from_db() -> Result<Option<HashMap<String, Entity>>> {
    let db_path = paths::db_path();
    if !db_path.exists() {
        return Ok(None);
    }

    let conn = match rusqlite::Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    ) {
        Ok(c) => c,
        Err(e) => {
            tracing::debug!(error = %e, "DB open failed, falling back to JSON");
            return Ok(None);
        }
    };

    match sqlite_db::load_graph_from_db(&conn) {
        Ok(Some(entities)) => {
            tracing::info!(entities = entities.len(), "loaded knowledge graph from DB");
            Ok(Some(entities))
        }
        Ok(None) => {
            tracing::debug!("DB has no graph data, falling back to JSON");
            Ok(None)
        }
        Err(e) => {
            tracing::debug!(error = %e, "DB graph load failed, falling back to JSON");
            Ok(None)
        }
    }
}

/// Load graph from `relations.json` with optional `raw/` description enrichment.
fn load_graph_from_json(data_dir: &Path) -> Result<KnowledgeGraph> {
    let relations_path = data_dir.join("relations.json");
    let raw = std::fs::read_to_string(&relations_path).map_err(InfraError::Io)?;
    let json_map: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(&raw).map_err(InfraError::Json)?;
    let mut entities = HashMap::new();
    for (key, value) in json_map {
        if !ENTITY_PREFIXES.iter().any(|prefix| key.starts_with(prefix)) {
            continue;
        }
        match serde_json::from_value::<Entity>(value) {
            Ok(mut entity) => {
                entity.id = key.clone();
                if entity.description.is_empty() && !entity.file_path.is_empty() {
                    let raw_dir = paths::raw_dir();
                    entity.description = extract_first_section(&raw_dir, &entity.file_path);
                }
                entities.insert(key, entity);
            }
            Err(e) => {
                tracing::warn!(key = %key, error = %e, "skipping malformed entity");
            }
        }
    }
    tracing::info!(
        entities = entities.len(),
        "loaded knowledge graph from JSON (fallback)"
    );
    Ok(KnowledgeGraph::from_entities(entities))
}

/// Reads the first paragraph of the first `##` section from a markdown file.
fn extract_first_section(raw_dir: &Path, file_path: &str) -> String {
    let full = raw_dir.join(file_path);
    let text = match std::fs::read_to_string(&full) {
        Ok(t) => t,
        Err(_) => return String::new(),
    };
    let mut in_section = false;
    let mut para = String::new();
    for line in text.lines() {
        if line.starts_with("## ") {
            if in_section {
                break;
            }
            in_section = true;
            continue;
        }
        if in_section {
            if line.starts_with('#') {
                break;
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                if !para.is_empty() {
                    break;
                }
            } else {
                if !para.is_empty() {
                    para.push(' ');
                }
                para.push_str(trimmed);
            }
        }
    }
    para
}
