use std::path::Path;

use crate::adapters::error::{InfraError, Result};
use crate::domain::types::Entity;
use crate::adapters::sqlite_db::Chunk;

/// Minimum character length for a section to be kept as a chunk.
const MIN_CHUNK_LEN: usize = 50;

/// Split a markdown file into semantic chunks.
///
/// Sections are delimited by `## ` headings.  The first section (before any
/// `## ` heading) is labelled "Overview".
///
/// Chunk IDs follow the format `{entity_id}-C{chunk_index:03}`.
pub fn chunk_markdown(file_path: &Path, entity_id: &str, entity: &Entity) -> Result<Vec<Chunk>> {
    let content = std::fs::read_to_string(file_path).map_err(InfraError::Io)?;

    let entity_type = if entity.r#type.is_empty() {
        "unknown".to_owned()
    } else {
        entity.r#type.clone()
    };
    let title = entity.title.clone();
    let metadata = build_metadata(entity_id, entity);

    let mut chunks = Vec::new();
    let mut current_section = "Overview".to_owned();
    let mut current_text: Vec<String> = Vec::new();
    let mut chunk_index: usize = 0;

    for line in content.split('\n') {
        if line.starts_with("## ") {
            // Flush the previous section.
            if !current_text.is_empty() {
                let text = current_text.join("\n");
                let trimmed = text.trim();
                if trimmed.len() > MIN_CHUNK_LEN {
                    chunks.push(Chunk {
                        id: format!("{entity_id}-C{chunk_index:03}"),
                        text: trimmed.to_owned(),
                        entity_id: entity_id.to_owned(),
                        entity_type: entity_type.clone(),
                        title: title.clone(),
                        section: current_section.clone(),
                        chunk_index: chunk_index as i64,
                        metadata: serde_json::to_string(&metadata)
                            .unwrap_or_else(|_| "{}".to_owned()),
                    });
                    chunk_index += 1;
                }
                current_text.clear();
            }
            current_section = line.strip_prefix("## ").unwrap_or(line).trim().to_owned();
        }
        current_text.push(line.to_owned());
    }

    // Flush the last section.
    if !current_text.is_empty() {
        let text = current_text.join("\n");
        let trimmed = text.trim();
        if trimmed.len() > MIN_CHUNK_LEN {
            chunks.push(Chunk {
                id: format!("{entity_id}-C{chunk_index:03}"),
                text: trimmed.to_owned(),
                entity_id: entity_id.to_owned(),
                entity_type: entity_type.clone(),
                title: title.clone(),
                section: current_section.clone(),
                chunk_index: chunk_index as i64,
                metadata: serde_json::to_string(&metadata)
                    .unwrap_or_else(|_| "{}".to_owned()),
            });
        }
    }

    Ok(chunks)
}

/// Build the metadata JSON value for a chunk, mirroring the Python
/// `_build_metadata` helper.
fn build_metadata(entity_id: &str, entity: &Entity) -> Metadata {
    Metadata {
        entity_id: entity_id.to_owned(),
        r#type: entity.r#type.clone(),
        category: entity.category.clone(),
        tags: entity.tags.clone(),
        solves: entity.relations.get("solves").cloned().unwrap_or_default(),
        solved_by: entity
            .relations
            .get("solved_by")
            .cloned()
            .unwrap_or_default(),
        enforces: entity
            .relations
            .get("enforces")
            .cloned()
            .unwrap_or_default(),
        violates: entity
            .relations
            .get("violates")
            .cloned()
            .unwrap_or_default(),
        related_to: entity
            .relations
            .get("related_to")
            .cloned()
            .unwrap_or_default(),
        source: entity.source.clone(),
        when_to_use: entity
            .context
            .get("when_to_use")
            .cloned()
            .unwrap_or_default(),
        symptoms: entity
            .context
            .get("symptoms")
            .cloned()
            .unwrap_or_default(),
        benefits: entity
            .context
            .get("benefits")
            .cloned()
            .unwrap_or_default(),
        drawbacks: entity
            .context
            .get("drawbacks")
            .cloned()
            .unwrap_or_default(),
    }
}

#[derive(Debug, Clone, serde::Serialize)]
struct Metadata {
    entity_id: String,
    r#type: String,
    category: String,
    tags: Vec<String>,
    solves: Vec<String>,
    solved_by: Vec<String>,
    enforces: Vec<String>,
    violates: Vec<String>,
    related_to: Vec<String>,
    source: serde_json::Value,
    when_to_use: Vec<String>,
    symptoms: Vec<String>,
    benefits: Vec<String>,
    drawbacks: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_entity() -> Entity {
        Entity {
            id: "DP-005".to_owned(),
            r#type: "pattern".to_owned(),
            title: "Singleton".to_owned(),
            description: String::new(),
            name: "Singleton".to_owned(),
            category: "creational".to_owned(),
            tags: vec![],
            relations: HashMap::new(),
            context: HashMap::new(),
            file_path: String::new(),
            source: serde_json::Value::Null,
        }
    }

    #[test]
    fn single_section_overview() {
        let md = "# Singleton\n\nThis is the singleton pattern. It ensures only one instance exists and provides a global point of access to it.";
        let dir = std::env::temp_dir().join("syntagma_test_chunker_overview");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("singleton.md");
        std::fs::write(&path, md).unwrap();

        let entity = make_entity();
        let chunks = chunk_markdown(&path, "DP-005", &entity).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].section, "Overview");
        assert_eq!(chunks[0].id, "DP-005-C000");
    }

    #[test]
    fn multiple_sections() {
        let md = "# Singleton\n\nSome intro text that is long enough to pass the minimum threshold.\n\n## Intent\n\nThe intent section describes what the pattern does and why you would use it.\n\n## Structure\n\nThe structure section describes the class diagram and relationships.";
        let dir = std::env::temp_dir().join("syntagma_test_chunker_multi");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("singleton.md");
        std::fs::write(&path, md).unwrap();

        let entity = make_entity();
        let chunks = chunk_markdown(&path, "DP-005", &entity).unwrap();
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].section, "Overview");
        assert_eq!(chunks[1].section, "Intent");
        assert_eq!(chunks[2].section, "Structure");
        assert_eq!(chunks[2].id, "DP-005-C002");
    }

    #[test]
    fn short_section_discarded() {
        let md = "# Test\n\nThis is long enough for the overview section to pass minimum threshold check.\n\n## Tiny\n\nhi";
        let dir = std::env::temp_dir().join("syntagma_test_chunker_short");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.md");
        std::fs::write(&path, md).unwrap();

        let entity = make_entity();
        let chunks = chunk_markdown(&path, "DP-005", &entity).unwrap();
        // "hi" is < 50 chars so it should be discarded.
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].section, "Overview");
    }
}
