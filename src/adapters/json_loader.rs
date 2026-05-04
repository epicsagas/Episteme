use std::path::Path;
use std::collections::HashMap;

use crate::domain::types::Entity;
use crate::domain::graph::KnowledgeGraph;
use crate::adapters::error::{InfraError, Result};
use crate::adapters::paths;

const ENTITY_PREFIXES: &[&str] = &["DP-", "RF-", "LAW-", "SMELL-"];

pub fn load_graph(data_dir: &Path) -> Result<KnowledgeGraph> {
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
            if in_section { break; }
            in_section = true;
            continue;
        }
        if in_section {
            if line.starts_with('#') { break; }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                if !para.is_empty() { break; }
            } else {
                if !para.is_empty() { para.push(' '); }
                para.push_str(trimmed);
            }
        }
    }
    para
}
