//! Data integrity tests for the knowledge graph.
//!
//! Validates that `meta/relations.json` is self-consistent:
//! - All forward relation targets resolve to existing entities
//! - All file_path values point to real files under `raw/`
//! - The derive_solved_by logic produces zero inconsistencies

use std::collections::HashMap;
use std::path::Path;

/// Minimal entity struct for deserialization — avoids importing crate internals.
#[derive(serde::Deserialize)]
struct Entity {
    #[allow(dead_code)]
    id: String,
    relations: HashMap<String, Vec<String>>,
    file_path: String,
}

fn load_relations() -> HashMap<String, Entity> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("meta/relations.json");
    let raw = std::fs::read_to_string(&path).expect("meta/relations.json must exist");
    let map: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(&raw).expect("valid JSON");

    let mut entities = HashMap::new();
    for (key, value) in map {
        if !key.starts_with("DP-")
            && !key.starts_with("RF-")
            && !key.starts_with("LAW-")
            && !key.starts_with("SMELL-")
        {
            continue;
        }
        match serde_json::from_value::<Entity>(value) {
            Ok(mut e) => {
                e.id = key.clone();
                entities.insert(key, e);
            }
            Err(_) => continue,
        }
    }
    entities
}

#[test]
fn all_forward_relation_targets_exist() {
    let entities = load_relations();
    let mut missing = Vec::new();

    for (eid, entity) in &entities {
        for (rel_type, targets) in &entity.relations {
            for target in targets {
                if !entities.contains_key(target) {
                    missing.push(format!("{eid} --{rel_type}--> {target}"));
                }
            }
        }
    }

    assert!(
        missing.is_empty(),
        "Dangling relation targets:\n{}",
        missing.join("\n")
    );
}

#[test]
fn all_file_paths_resolve() {
    let entities = load_relations();
    let raw_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("raw");
    let mut broken = Vec::new();

    for (eid, entity) in &entities {
        if entity.file_path.is_empty() {
            continue;
        }
        let full_path = raw_dir.join(&entity.file_path);
        if !full_path.exists() {
            broken.push(format!("{eid}: {} (not found)", entity.file_path));
        }
    }

    assert!(
        broken.is_empty(),
        "Broken file_path entries:\n{}",
        broken.join("\n")
    );
}

#[test]
fn solved_by_not_stored_in_data() {
    let entities = load_relations();

    // solved_by is now derived at load time from forward "solves" edges.
    // The data file must NOT contain solved_by — if present, it would be
    // overwritten at runtime but signals a data authoring mistake.
    let mut violations = Vec::new();
    for (eid, entity) in &entities {
        if entity.relations.contains_key("solved_by") {
            violations.push(eid.clone());
        }
    }

    assert!(
        violations.is_empty(),
        "These entities still contain solved_by in data (should be derived at runtime):\n{}",
        violations.join(", ")
    );
}
