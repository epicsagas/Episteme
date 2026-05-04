//! Search domain: hybrid RAG search, keyword fallback, and result formatting.

use std::sync::Mutex;

use crate::adapters::constants;
use crate::adapters::search_engines::{self, SearchResult};
use crate::domain::graph::KnowledgeGraph;
use crate::ports::embeddings::EmbeddingProvider;

/// Search knowledge graph entities.
///
/// When a RAG database is attached, uses hybrid search (FTS5 + semantic with RRF fusion).
/// Uses problem_mapper to auto-detect entity types when not provided.
/// Falls back to keyword matching over graph entities when no RAG DB is available.
pub fn search_knowledge(
    graph: &KnowledgeGraph,
    db: Option<&Mutex<rusqlite::Connection>>,
    embedding_provider: Option<&dyn EmbeddingProvider>,
    query: &str,
    limit: Option<usize>,
    entity_type: Option<&str>,
) -> serde_json::Value {
    let limit = limit
        .unwrap_or(constants::DEFAULT_SEARCH_LIMIT)
        .clamp(1, constants::MAX_SEARCH_LIMIT);

    let query_lower = query.to_lowercase();
    let terms: Vec<&str> = query_lower.split_whitespace().collect();

    if terms.is_empty() {
        return serde_json::json!({"results": [], "count": 0});
    }

    // Auto-detect entity types via problem_mapper when not specified
    let entity_types: Vec<String> = match entity_type {
        Some(et) => vec![et.to_owned()],
        None => {
            use crate::domain::problem_mapper::map_problem_to_entity_types;
            map_problem_to_entity_types(query)
                .into_iter()
                .map(|(t, _)| t)
                .collect()
        }
    };

    // Try hybrid RAG search with entity type detection
    if let (Some(db_mutex), Some(provider)) = (db, embedding_provider)
        && let Ok(conn) = db_mutex.lock() {
            // Multi-type parallel search with RRF merge (mirrors Python behavior)
            if entity_types.len() >= 2 {
                let mut merged: Vec<SearchResult> = Vec::new();
                for etype in &entity_types {
                    if let Ok(rag_results) = search_engines::hybrid_search(
                        &conn,
                        provider,
                        query,
                        limit,
                        Some(etype),
                        None,
                    ) {
                        merged.extend(rag_results);
                    }
                }
                if !merged.is_empty() {
                    // Deduplicate by chunk_id and re-rank by score
                    let mut seen = std::collections::HashSet::new();
                    merged.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
                    merged.retain(|r| seen.insert(r.chunk_id.clone()));
                    merged.truncate(limit);
                    return serde_json::json!({
                        "results": rag_results_to_json(graph, &merged),
                        "count": merged.len(),
                    });
                }
            } else {
                // Single or no entity type filter
                let etype_filter = entity_types.first().map(|s| s.as_str());
                if let Ok(rag_results) = search_engines::hybrid_search(
                    &conn,
                    provider,
                    query,
                    limit,
                    etype_filter,
                    None,
                )
                    && !rag_results.is_empty() {
                        return serde_json::json!({
                            "results": rag_results_to_json(graph, &rag_results),
                            "count": rag_results.len(),
                        });
                    }
            }
        }

    // Fallback: keyword search over graph entities
    let etype = entity_types.first().map(|s| s.as_str());
    let results = keyword_search(graph, &terms, etype, limit);

    serde_json::json!({
        "results": results,
        "count": results.len(),
    })
}

/// Convert RAG search results to JSON values.
pub fn rag_results_to_json(
    graph: &KnowledgeGraph,
    results: &[SearchResult],
) -> Vec<serde_json::Value> {
    results
        .iter()
        .map(|r| {
            let entity = graph.get_entity(&r.entity_id);
            serde_json::json!({
                "entity_id": r.entity_id,
                "title": if r.title.is_empty() {
                    entity.map(|e| e.title.as_str()).unwrap_or("")
                } else {
                    &r.title
                },
                "type": entity.map(|e| e.r#type.as_str()).unwrap_or(&r.entity_type),
                "category": entity.map(|e| e.category.as_str()).unwrap_or(""),
                "score": format!("{:.4}", r.score),
                "section": r.section,
                "text": r.text,
            })
        })
        .collect()
}

/// Internal keyword search over graph entities.
///
/// Scores entities by counting term matches across their text fields.
pub fn keyword_search(
    graph: &KnowledgeGraph,
    terms: &[&str],
    entity_type: Option<&str>,
    limit: usize,
) -> Vec<serde_json::Value> {
    let entity_ids = graph.all_entity_ids();
    let ids_ref: Vec<&str> = entity_ids.iter().map(|s| s.as_str()).collect();
    let batch = graph.get_entities_batch(&ids_ref);

    let mut results: Vec<(String, usize)> = Vec::new();

    for (id, entity) in &batch {
        // Filter by entity type if requested
        if let Some(etype) = entity_type
            && entity.r#type != etype {
                continue;
            }

        // Build a searchable text from the entity
        let mut text_parts = vec![
            entity.title.to_lowercase(),
            entity.name.to_lowercase(),
            entity.r#type.to_lowercase(),
            entity.category.to_lowercase(),
        ];
        for tag in &entity.tags {
            text_parts.push(tag.to_lowercase());
        }
        for (key, values) in &entity.context {
            text_parts.push(key.to_lowercase());
            for v in values {
                text_parts.push(v.to_lowercase());
            }
        }
        let text = text_parts.join(" ");

        let score = terms.iter().filter(|term| text.contains(*term)).count();
        if score > 0 {
            results.push((id.clone(), score));
        }
    }

    // Sort by score descending
    results.sort_by_key(|b| std::cmp::Reverse(b.1));
    results.truncate(limit);

    results
        .into_iter()
        .map(|(id, score)| {
            let entity = batch.get(&id);
            serde_json::json!({
                "entity_id": id,
                "title": entity.map(|e| e.title.as_str()).unwrap_or(""),
                "type": entity.map(|e| e.r#type.as_str()).unwrap_or(""),
                "category": entity.map(|e| e.category.as_str()).unwrap_or(""),
                "score": score,
            })
        })
        .collect()
}
