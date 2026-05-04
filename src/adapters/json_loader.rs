use std::path::Path;
use std::collections::HashMap;

use crate::domain::types::Entity;
use crate::domain::graph::KnowledgeGraph;
use crate::adapters::error::{InfraError, Result};

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
                entities.insert(key, entity);
            }
            Err(e) => {
                tracing::warn!(key = %key, error = %e, "skipping malformed entity");
            }
        }
    }
    Ok(KnowledgeGraph::from_entities(entities))
}
