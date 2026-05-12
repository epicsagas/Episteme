//! Data integrity tests for the knowledge graph.
//!
//! Validates that `meta/relations.json` is self-consistent:
//! - All forward relation targets resolve to existing entities
//! - All file_path values point to real files under `raw/`
//! - The derive_solved_by logic produces zero inconsistencies
//! - Semantic correctness of key knowledge relationships
//! - No broken markdown extraction artifacts
//! - Content completeness for entities

use std::collections::HashMap;
use std::path::Path;

/// Minimal entity struct for deserialization — avoids importing crate internals.
#[derive(serde::Deserialize)]
struct Entity {
    #[allow(dead_code)]
    id: String,
    #[serde(default)]
    r#type: String,
    relations: HashMap<String, Vec<String>>,
    file_path: String,
    #[serde(default)]
    context: HashMap<String, Vec<String>>,
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

// ---------------------------------------------------------------------------
// Semantic validation tests
// ---------------------------------------------------------------------------

/// Helper: assert that entity `from_id` has a `rel_type` edge to `to_id`.
fn assert_has_relation(
    entities: &HashMap<String, Entity>,
    from_id: &str,
    rel_type: &str,
    to_id: &str,
) {
    let entity = entities
        .get(from_id)
        .unwrap_or_else(|| panic!("entity {from_id} not found"));
    let targets = entity
        .relations
        .get(rel_type)
        .unwrap_or_else(|| panic!("{from_id} has no '{rel_type}' relation"));
    assert!(
        targets.contains(&to_id.to_string()),
        "{from_id} --{rel_type}--> {to_id} not found (has: {})",
        targets.join(", ")
    );
}

/// Helper: assert that entity `from_id` does NOT have a `rel_type` edge to `to_id`.
fn assert_not_has_relation(
    entities: &HashMap<String, Entity>,
    from_id: &str,
    rel_type: &str,
    to_id: &str,
) {
    if let Some(entity) = entities.get(from_id) {
        if let Some(targets) = entity.relations.get(rel_type) {
            assert!(
                !targets.contains(&to_id.to_string()),
                "{from_id} should NOT have --{rel_type}--> {to_id}"
            );
        }
    }
}

// --- R1: Pattern→Smell solves edges ---

#[test]
fn strategy_solves_switch_statements() {
    let entities = load_relations();
    assert_has_relation(&entities, "DP-020", "solves", "SMELL-06");
}

#[test]
fn state_solves_switch_statements() {
    let entities = load_relations();
    assert_has_relation(&entities, "DP-019", "solves", "SMELL-06");
}

#[test]
fn facade_solves_long_method_and_god_object() {
    let entities = load_relations();
    // DP-010 = Facade, SMELL-01 = Long Method, SMELL-21 = God Object
    assert_has_relation(&entities, "DP-010", "solves", "SMELL-01");
    assert_has_relation(&entities, "DP-010", "solves", "SMELL-21");
}

#[test]
fn template_method_solves_primitive_obsession() {
    // DP-021 = Template Method, SMELL-03 = Primitive Obsession
    let entities = load_relations();
    assert_has_relation(&entities, "DP-021", "solves", "SMELL-03");
}

#[test]
fn decorator_solves_primitive_obsession() {
    // DP-009 = Decorator, SMELL-03 = Primitive Obsession
    let entities = load_relations();
    assert_has_relation(&entities, "DP-009", "solves", "SMELL-03");
}

#[test]
fn observer_solves_inappropriate_intimacy() {
    let entities = load_relations();
    assert_has_relation(&entities, "DP-018", "solves", "SMELL-19");
}

#[test]
fn mediator_solves_inappropriate_intimacy() {
    let entities = load_relations();
    assert_has_relation(&entities, "DP-016", "solves", "SMELL-19");
}

#[test]
fn command_solves_switch_statements() {
    let entities = load_relations();
    assert_has_relation(&entities, "DP-014", "solves", "SMELL-06");
}

#[test]
fn visitor_solves_switch_statements() {
    let entities = load_relations();
    assert_has_relation(&entities, "DP-022", "solves", "SMELL-06");
}

#[test]
fn composite_solves_switch_statements() {
    // DP-008 = Composite, SMELL-06 = Switch Statements
    let entities = load_relations();
    assert_has_relation(&entities, "DP-008", "solves", "SMELL-06");
}

// --- R3: No broken extraction artifacts ---

#[test]
fn no_star_star_in_context_fields() {
    let entities = load_relations();
    let mut broken = Vec::new();
    for (eid, entity) in &entities {
        for (field, values) in &entity.context {
            for v in values {
                if v == "**" || v == "*" {
                    broken.push(format!("{eid}.context.{field} = \"{v}\""));
                }
            }
        }
    }
    assert!(
        broken.is_empty(),
        "Broken markdown artifacts (** or *) found:\n{}",
        broken.join("\n")
    );
}

#[test]
fn no_empty_strings_in_context_fields() {
    let entities = load_relations();
    let mut broken = Vec::new();
    for (eid, entity) in &entities {
        for (field, values) in &entity.context {
            for v in values {
                if v.is_empty() {
                    broken.push(format!("{eid}.context.{field} = \"\""));
                }
            }
        }
    }
    assert!(
        broken.is_empty(),
        "Empty strings in context fields:\n{}",
        broken.join("\n")
    );
}

#[test]
fn no_truncated_when_to_use() {
    let entities = load_relations();
    let patterns: Vec<_> = entities
        .iter()
        .filter(|(_, e)| e.r#type == "pattern")
        .collect();
    let mut broken = Vec::new();
    for (eid, entity) in &patterns {
        if let Some(wtu) = entity.context.get("when_to_use") {
            for v in wtu {
                if v.ends_with(' ') && v.len() < 120 {
                    broken.push(format!("{eid}: \"{v}\""));
                }
                if v.ends_with(':') && v.len() < 30 {
                    broken.push(format!("{eid}: \"{v}\""));
                }
            }
        }
    }
    assert!(
        broken.is_empty(),
        "Truncated when_to_use entries:\n{}",
        broken.join("\n")
    );
}

// --- R2: Refactoring content completeness ---

#[test]
fn all_refactorings_have_when_to_use() {
    let entities = load_relations();
    let mut empty = Vec::new();
    for (eid, entity) in &entities {
        if eid.starts_with("RF-") {
            match entity.context.get("when_to_use") {
                Some(v) if !v.is_empty() => {}
                _ => empty.push(eid.clone()),
            }
        }
    }
    assert!(
        empty.is_empty(),
        "Refactorings with empty when_to_use:\n{}",
        empty.join(", ")
    );
}

#[test]
fn all_refactorings_have_benefits() {
    let entities = load_relations();
    let mut empty = Vec::new();
    for (eid, entity) in &entities {
        if eid.starts_with("RF-") {
            match entity.context.get("benefits") {
                Some(v) if !v.is_empty() => {}
                _ => empty.push(eid.clone()),
            }
        }
    }
    assert!(
        empty.is_empty(),
        "Refactorings with empty benefits:\n{}",
        empty.join(", ")
    );
}

#[test]
fn all_refactorings_have_symptoms() {
    let entities = load_relations();
    let mut empty = Vec::new();
    for (eid, entity) in &entities {
        if eid.starts_with("RF-") {
            match entity.context.get("symptoms") {
                Some(v) if !v.is_empty() => {}
                _ => empty.push(eid.clone()),
            }
        }
    }
    assert!(
        empty.is_empty(),
        "Refactorings with empty symptoms:\n{}",
        empty.join(", ")
    );
}

// --- R4: Correct smell→refactoring mappings ---

#[test]
fn data_clumps_solved_by_introduce_parameter_object() {
    // RF-043 = Introduce Parameter Object, SMELL-05 = Data Clumps
    let entities = load_relations();
    assert_has_relation(&entities, "RF-043", "solves", "SMELL-05");
}

#[test]
fn data_clumps_not_solved_by_self_encapsulate_field() {
    // RF-032 = Self Encapsulate Field should NOT solve Data Clumps
    let entities = load_relations();
    assert_not_has_relation(&entities, "RF-032", "solves", "SMELL-05");
}

#[test]
fn data_clumps_not_solved_by_encapsulate_collection() {
    // RF-023 = Encapsulate Collection should NOT solve Data Clumps
    let entities = load_relations();
    assert_not_has_relation(&entities, "RF-023", "solves", "SMELL-05");
}

#[test]
fn god_object_solved_by_extract_class() {
    // RF-010 = Extract Class, SMELL-21 = God Object
    let entities = load_relations();
    assert_has_relation(&entities, "RF-010", "solves", "SMELL-21");
}

#[test]
fn switch_statements_solved_by_replace_conditional_polymorphism() {
    // RF-039 = Replace Conditional With Polymorphism, SMELL-06 = Switch Statements
    let entities = load_relations();
    assert_has_relation(&entities, "RF-039", "solves", "SMELL-06");
}

#[test]
fn long_method_solved_by_extract_method() {
    // RF-001 = Extract Method, SMELL-01 = Long Method
    let entities = load_relations();
    assert_has_relation(&entities, "RF-001", "solves", "SMELL-01");
}

#[test]
fn feature_envy_solved_by_move_method() {
    // RF-016 = Move Method, SMELL-18 = Feature Envy
    let entities = load_relations();
    assert_has_relation(&entities, "RF-016", "solves", "SMELL-18");
}

// --- R6: Edge type distribution ---

#[test]
fn related_to_is_less_than_55_percent() {
    let entities = load_relations();
    let mut total_edges = 0usize;
    let mut related_to_count = 0usize;
    for entity in entities.values() {
        for (rel_type, targets) in &entity.relations {
            let count = targets.len();
            total_edges += count;
            if rel_type == "related_to" {
                related_to_count += count;
            }
        }
    }
    let pct = (related_to_count as f64 / total_edges as f64) * 100.0;
    assert!(
        pct < 55.0,
        "related_to is {pct:.1}% of all edges ({related_to_count}/{total_edges}), should be < 55%"
    );
}

// --- R1: Pattern solves coverage ---

#[test]
fn at_least_15_patterns_have_solves_edges() {
    let entities = load_relations();
    let with_solves = entities
        .iter()
        .filter(|(eid, e)| eid.starts_with("DP-") && e.r#type == "pattern")
        .filter(|(_, e)| !e.relations.get("solves").map_or(true, |v| v.is_empty()))
        .count();
    assert!(
        with_solves >= 15,
        "Only {with_solves} patterns have solves edges, expected at least 15"
    );
}

// --- R6: Pattern→Law enforces edges ---

#[test]
fn strategy_enforces_ocp() {
    let entities = load_relations();
    assert_has_relation(&entities, "DP-020", "enforces", "LAW-042-O");
}

#[test]
fn decorator_enforces_ocp() {
    // DP-009 = Decorator, LAW-042-O = OCP
    let entities = load_relations();
    assert_has_relation(&entities, "DP-009", "enforces", "LAW-042-O");
}
