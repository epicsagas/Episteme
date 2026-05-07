//! Graph domain: entity lookup, neighbor traversal, and shortest-path queries.

use crate::domain::graph::KnowledgeGraph;
use crate::domain::summarizer::{DetailLevel, summarize_entity};

/// Get detailed information about a single entity.
pub fn get_entity(
    graph: &KnowledgeGraph,
    entity_id: &str,
    detail_level: Option<&str>,
) -> serde_json::Value {
    let entity = match graph.get_entity(entity_id) {
        Some(e) => e,
        None => {
            return serde_json::json!({
                "error": format!("Entity '{}' not found.", entity_id)
            });
        }
    };

    let level = match detail_level.unwrap_or("summary") {
        "minimal" => DetailLevel::Minimal,
        "detailed" => DetailLevel::Detailed,
        "full" => DetailLevel::Full,
        _ => DetailLevel::Summary,
    };

    let entity_json = serde_json::to_value(entity).unwrap_or(serde_json::Value::Null);
    summarize_entity(&entity_json, level)
}

/// Get entities related to a given entity, optionally filtered by relation type.
pub fn get_neighbors(
    graph: &KnowledgeGraph,
    entity_id: &str,
    relation_type: Option<&str>,
) -> serde_json::Value {
    if graph.get_entity(entity_id).is_none() {
        return serde_json::json!({
            "error": format!("Entity '{}' not found.", entity_id)
        });
    }

    let neighbor_ids = graph.get_neighbors(entity_id, relation_type);
    let ids_ref: Vec<&str> = neighbor_ids.iter().map(|s| s.as_str()).collect();
    let entities_map = graph.get_entities_batch(&ids_ref);

    let neighbors: Vec<serde_json::Value> = neighbor_ids
        .iter()
        .filter_map(|nid| {
            entities_map.get(nid).map(|e| {
                serde_json::json!({
                    "id": nid,
                    "title": e.title,
                    "type": e.r#type,
                })
            })
        })
        .collect();

    serde_json::json!({
        "entity_id": entity_id,
        "relation_type": relation_type,
        "neighbors": neighbors,
    })
}

/// Find the shortest path between two entities in the knowledge graph.
pub fn find_path(
    graph: &KnowledgeGraph,
    from_id: &str,
    to_id: &str,
    max_depth: Option<usize>,
) -> serde_json::Value {
    let max_depth = max_depth.unwrap_or(5);

    match graph.find_shortest_path(from_id, to_id, max_depth) {
        Some(path) => {
            let ids_ref: Vec<&str> = path.iter().map(|s| s.as_str()).collect();
            let entities_map = graph.get_entities_batch(&ids_ref);

            let labeled: Vec<serde_json::Value> = path
                .iter()
                .map(|pid| {
                    let title = entities_map
                        .get(pid.as_str())
                        .map(|e| e.title.as_str())
                        .unwrap_or("");
                    serde_json::json!({
                        "id": pid,
                        "title": title,
                    })
                })
                .collect();

            serde_json::json!({
                "from": from_id,
                "to": to_id,
                "length": path.len() - 1,
                "path": labeled,
            })
        }
        None => serde_json::json!({
            "error": format!("No path found between '{}' and '{}'.", from_id, to_id)
        }),
    }
}
