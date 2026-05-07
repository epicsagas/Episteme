//! Search domain: hybrid RAG search, keyword fallback, and result formatting.

use std::collections::HashSet;
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
        && let Ok(conn) = db_mutex.lock()
    {
        // Multi-type parallel search with RRF merge (mirrors Python behavior)
        if entity_types.len() >= 2 {
            let mut merged: Vec<SearchResult> = Vec::new();
            for etype in &entity_types {
                if let Ok(rag_results) =
                    search_engines::hybrid_search(&conn, provider, query, limit, Some(etype), None)
                {
                    merged.extend(rag_results);
                }
            }
            if !merged.is_empty() {
                merged.sort_by(|a, b| {
                    b.score
                        .partial_cmp(&a.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                inject_intent_synonyms(graph, query, &mut merged, limit);
                expand_with_related_entities(graph, &mut merged, limit);
                let deduped = entity_dedup(merged, limit);
                return serde_json::json!({
                    "results": rag_results_to_json(graph, &deduped),
                    "count": deduped.len(),
                });
            }
        } else {
            // Single or no entity type filter
            let etype_filter = entity_types.first().map(|s| s.as_str());
            if let Ok(rag_results) =
                search_engines::hybrid_search(&conn, provider, query, limit, etype_filter, None)
                && !rag_results.is_empty()
            {
                let mut sorted = rag_results;
                sorted.sort_by(|a, b| {
                    b.score
                        .partial_cmp(&a.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                inject_intent_synonyms(graph, query, &mut sorted, limit);
                expand_with_related_entities(graph, &mut sorted, limit);
                let deduped = entity_dedup(sorted, limit);
                return serde_json::json!({
                    "results": rag_results_to_json(graph, &deduped),
                    "count": deduped.len(),
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

    // Score = (total_match_count << 8) | title_match_count, so title hits break ties.
    let mut results: Vec<(String, usize)> = Vec::new();

    for (id, entity) in &batch {
        // Filter by entity type if requested
        if let Some(etype) = entity_type
            && entity.r#type != etype
        {
            continue;
        }

        let title_lower = entity.title.to_lowercase();
        let name_lower = entity.name.to_lowercase();

        // Build a searchable text from the entity
        let mut text_parts = vec![
            title_lower.clone(),
            name_lower.clone(),
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

        let total_matches = terms.iter().filter(|term| text.contains(*term)).count();
        if total_matches == 0 {
            continue;
        }

        // Title hits count separately for tie-breaking (shifted into high bits).
        let title_matches = terms
            .iter()
            .filter(|term| title_lower.contains(*term) || name_lower.contains(*term))
            .count();
        let composite_score = (total_matches << 8) | title_matches.min(255);
        results.push((id.clone(), composite_score));
    }

    // Sort by composite score descending (higher total matches first, title matches break ties)
    results.sort_by_key(|b| std::cmp::Reverse(b.1));
    results.truncate(limit);

    results
        .into_iter()
        .map(|(id, composite_score)| {
            let entity = batch.get(&id);
            // Recover the original total match count from the composite score.
            let display_score = composite_score >> 8;
            serde_json::json!({
                "entity_id": id,
                "title": entity.map(|e| e.title.as_str()).unwrap_or(""),
                "type": entity.map(|e| e.r#type.as_str()).unwrap_or(""),
                "category": entity.map(|e| e.category.as_str()).unwrap_or(""),
                "score": display_score,
            })
        })
        .collect()
}

// Injected synonyms rank just below the best organic result so they appear
// without displacing direct matches.
const SYNONYM_SCORE_RATIO: f64 = 0.95;
// Graph-expanded remedies rank just below the lowest smell result.
const EXPANSION_SCORE_RATIO: f64 = 0.9;

/// Inject entities from intent synonym matching when the query matches
/// curated abstract intent phrases (e.g. "flexible" -> Strategy pattern).
fn inject_intent_synonyms(
    graph: &KnowledgeGraph,
    query: &str,
    results: &mut Vec<SearchResult>,
    limit: usize,
) {
    use crate::domain::problem_mapper::lookup_intent_synonyms;

    let synonym_ids = lookup_intent_synonyms(query);
    if synonym_ids.is_empty() {
        return;
    }

    let existing_ids: HashSet<String> = results.iter().map(|r| r.entity_id.clone()).collect();
    let top_score = results.first().map(|r| r.score).unwrap_or(0.5);

    let mut injected = 0usize;
    for entity_id in &synonym_ids {
        if injected >= 2 {
            break;
        }
        if existing_ids.contains(entity_id) {
            continue;
        }
        let Some(entity) = graph.get_entity(entity_id) else {
            continue;
        };

        results.push(SearchResult {
            chunk_id: format!("synonym_{}", entity_id),
            entity_id: entity_id.clone(),
            entity_type: entity.r#type.clone(),
            title: entity.title.clone(),
            section: "Intent Match".to_owned(),
            text: entity.title.clone(),
            metadata_json: String::new(),
            score: top_score * SYNONYM_SCORE_RATIO,
            similarity: 0.0,
            keyword_rank: None,
            semantic_rank: None,
        });
        injected += 1;
    }

    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results.truncate(limit);
}

/// When smell entities dominate the top results, auto-expand with related
/// refactoring remedies from the knowledge graph via `solved_by` relations.
fn expand_with_related_entities(
    graph: &KnowledgeGraph,
    results: &mut Vec<SearchResult>,
    limit: usize,
) {
    let top_smells: Vec<&SearchResult> = results
        .iter()
        .take(3)
        .filter(|r| r.entity_type == "smell")
        .collect();

    if top_smells.is_empty() {
        return;
    }

    let lowest_smell_score = top_smells
        .iter()
        .map(|r| r.score)
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(0.0);

    let existing_ids: HashSet<String> = results.iter().map(|r| r.entity_id.clone()).collect();
    let mut expanded: Vec<SearchResult> = Vec::new();

    for smell in &top_smells {
        if expanded.len() >= 2 {
            break;
        }

        let mut neighbor_ids = graph.get_neighbors(&smell.entity_id, Some("solved_by"));
        if neighbor_ids.is_empty() {
            neighbor_ids = graph
                .get_neighbors(&smell.entity_id, None)
                .into_iter()
                .filter(|id| id.starts_with("RF-"))
                .collect();
        }

        for neighbor_id in neighbor_ids {
            if expanded.len() >= 2 {
                break;
            }
            if existing_ids.contains(&neighbor_id)
                || expanded.iter().any(|r| r.entity_id == neighbor_id)
            {
                continue;
            }

            let Some(entity) = graph.get_entity(&neighbor_id) else {
                continue;
            };

            expanded.push(SearchResult {
                chunk_id: format!("expanded_{}", neighbor_id),
                entity_id: neighbor_id.clone(),
                entity_type: entity.r#type.clone(),
                title: entity.title.clone(),
                section: "Related Solution".to_owned(),
                text: entity.title.clone(),
                metadata_json: String::new(),
                score: lowest_smell_score * EXPANSION_SCORE_RATIO,
                similarity: 0.0,
                keyword_rank: None,
                semantic_rank: None,
            });
        }
    }

    results.extend(expanded);
    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results.truncate(limit);
}

/// Deduplicate RAG results by entity_id, keeping the highest-scoring chunk
/// per entity, then truncate to `limit`.
///
/// Assumes `results` is already sorted by score descending.
fn entity_dedup(results: Vec<SearchResult>, limit: usize) -> Vec<SearchResult> {
    let mut seen = std::collections::HashSet::new();
    results
        .into_iter()
        .filter(|r| seen.insert(r.entity_id.clone()))
        .take(limit)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::graph::tests::{blank_entity, build_graph_from_entities};

    #[test]
    fn expand_adds_refactorings_for_smell_results() {
        let mut smell = blank_entity("SMELL-01");
        smell.title = "Long Method".to_owned();
        smell.r#type = "smell".to_owned();
        smell
            .relations
            .insert("solved_by".to_owned(), vec!["RF-001".to_owned()]);

        let mut rf = blank_entity("RF-001");
        rf.title = "Extract Method".to_owned();
        rf.r#type = "refactoring".to_owned();

        let graph = build_graph_from_entities(vec![smell, rf]);

        let mut results = vec![SearchResult {
            chunk_id: "chunk_1".to_owned(),
            text: "A long method is a method that is too long".to_owned(),
            entity_id: "SMELL-01".to_owned(),
            entity_type: "smell".to_owned(),
            title: "Long Method".to_owned(),
            section: "Description".to_owned(),
            metadata_json: String::new(),
            score: 0.85,
            similarity: 0.85,
            keyword_rank: None,
            semantic_rank: None,
        }];

        expand_with_related_entities(&graph, &mut results, 10);

        assert_eq!(results.len(), 2);
        let expanded = results
            .iter()
            .find(|r| r.entity_id == "RF-001")
            .expect("should find RF-001");
        assert_eq!(expanded.title, "Extract Method");
        assert_eq!(expanded.section, "Related Solution");
        assert_eq!(expanded.chunk_id, "expanded_RF-001");
        assert!((expanded.score - 0.85 * EXPANSION_SCORE_RATIO).abs() < f64::EPSILON);
        assert_eq!(expanded.similarity, 0.0);
    }

    #[test]
    fn expand_skips_already_present_entities() {
        let mut smell = blank_entity("SMELL-01");
        smell.title = "Long Method".to_owned();
        smell.r#type = "smell".to_owned();
        smell
            .relations
            .insert("solved_by".to_owned(), vec!["RF-001".to_owned()]);

        let mut rf = blank_entity("RF-001");
        rf.title = "Extract Method".to_owned();
        rf.r#type = "refactoring".to_owned();

        let graph = build_graph_from_entities(vec![smell, rf]);

        let mut results = vec![
            SearchResult {
                chunk_id: "chunk_1".to_owned(),
                text: "A long method".to_owned(),
                entity_id: "SMELL-01".to_owned(),
                entity_type: "smell".to_owned(),
                title: "Long Method".to_owned(),
                section: "Description".to_owned(),
                metadata_json: String::new(),
                score: 0.85,
                similarity: 0.85,
                keyword_rank: None,
                semantic_rank: None,
            },
            SearchResult {
                chunk_id: "chunk_2".to_owned(),
                text: "Extract Method".to_owned(),
                entity_id: "RF-001".to_owned(),
                entity_type: "refactoring".to_owned(),
                title: "Extract Method".to_owned(),
                section: "Description".to_owned(),
                metadata_json: String::new(),
                score: 0.80,
                similarity: 0.80,
                keyword_rank: None,
                semantic_rank: None,
            },
        ];

        expand_with_related_entities(&graph, &mut results, 10);

        assert_eq!(results.len(), 2, "should not add duplicates");
    }

    #[test]
    fn expand_limits_to_two_entities() {
        let mut smell = blank_entity("SMELL-01");
        smell.title = "Long Method".to_owned();
        smell.r#type = "smell".to_owned();
        smell.relations.insert(
            "solved_by".to_owned(),
            vec![
                "RF-001".to_owned(),
                "RF-002".to_owned(),
                "RF-003".to_owned(),
            ],
        );

        let mut rf1 = blank_entity("RF-001");
        rf1.title = "Extract Method".to_owned();
        rf1.r#type = "refactoring".to_owned();
        let mut rf2 = blank_entity("RF-002");
        rf2.title = "Decompose Conditional".to_owned();
        rf2.r#type = "refactoring".to_owned();
        let mut rf3 = blank_entity("RF-003");
        rf3.title = "Replace Temp with Query".to_owned();
        rf3.r#type = "refactoring".to_owned();

        let graph = build_graph_from_entities(vec![smell, rf1, rf2, rf3]);

        let mut results = vec![SearchResult {
            chunk_id: "chunk_1".to_owned(),
            text: "Long method smell".to_owned(),
            entity_id: "SMELL-01".to_owned(),
            entity_type: "smell".to_owned(),
            title: "Long Method".to_owned(),
            section: "Description".to_owned(),
            metadata_json: String::new(),
            score: 0.90,
            similarity: 0.90,
            keyword_rank: None,
            semantic_rank: None,
        }];

        expand_with_related_entities(&graph, &mut results, 10);

        let expanded_count = results
            .iter()
            .filter(|r| r.chunk_id.starts_with("expanded_"))
            .count();
        assert_eq!(expanded_count, 2, "should add at most 2 expanded results");
    }

    #[test]
    fn expand_does_nothing_for_non_smell_results() {
        let mut dp = blank_entity("DP-001");
        dp.title = "Singleton".to_owned();
        dp.r#type = "pattern".to_owned();

        let graph = build_graph_from_entities(vec![dp]);

        let mut results = vec![SearchResult {
            chunk_id: "chunk_1".to_owned(),
            text: "Singleton pattern".to_owned(),
            entity_id: "DP-001".to_owned(),
            entity_type: "pattern".to_owned(),
            title: "Singleton".to_owned(),
            section: "Description".to_owned(),
            metadata_json: String::new(),
            score: 0.90,
            similarity: 0.90,
            keyword_rank: None,
            semantic_rank: None,
        }];

        expand_with_related_entities(&graph, &mut results, 10);

        assert_eq!(results.len(), 1);
    }

    #[test]
    fn expand_uses_fallback_when_solved_by_empty() {
        let mut smell = blank_entity("SMELL-01");
        smell.title = "Long Method".to_owned();
        smell.r#type = "smell".to_owned();
        smell
            .relations
            .insert("related_to".to_owned(), vec!["RF-001".to_owned()]);

        let mut rf = blank_entity("RF-001");
        rf.title = "Extract Method".to_owned();
        rf.r#type = "refactoring".to_owned();

        let graph = build_graph_from_entities(vec![smell, rf]);

        let mut results = vec![SearchResult {
            chunk_id: "chunk_1".to_owned(),
            text: "Long method".to_owned(),
            entity_id: "SMELL-01".to_owned(),
            entity_type: "smell".to_owned(),
            title: "Long Method".to_owned(),
            section: "Description".to_owned(),
            metadata_json: String::new(),
            score: 0.90,
            similarity: 0.90,
            keyword_rank: None,
            semantic_rank: None,
        }];

        expand_with_related_entities(&graph, &mut results, 10);

        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|r| r.entity_id == "RF-001"));
    }

    #[test]
    fn inject_intent_synonyms_adds_matching_entities() {
        let mut dp = blank_entity("DP-020");
        dp.title = "Strategy".to_owned();
        dp.r#type = "pattern".to_owned();

        let mut law = blank_entity("LAW-042");
        law.title = "Open/Closed Principle".to_owned();
        law.r#type = "law".to_owned();

        let graph = build_graph_from_entities(vec![dp, law]);

        let mut results = vec![SearchResult {
            chunk_id: "c1".to_owned(),
            text: "Some pattern".to_owned(),
            entity_id: "DP-001".to_owned(),
            entity_type: "pattern".to_owned(),
            title: "Abstract Factory".to_owned(),
            section: "Overview".to_owned(),
            metadata_json: String::new(),
            score: 0.9,
            similarity: 0.9,
            keyword_rank: None,
            semantic_rank: None,
        }];

        inject_intent_synonyms(&graph, "flexible code", &mut results, 10);

        let injected: Vec<&SearchResult> = results
            .iter()
            .filter(|r| r.chunk_id.starts_with("synonym_"))
            .collect();
        assert_eq!(injected.len(), 2);
        assert_eq!(injected[0].section, "Intent Match");
        assert!((injected[0].score - 0.9 * SYNONYM_SCORE_RATIO).abs() < f64::EPSILON);
    }

    #[test]
    fn inject_intent_synonyms_caps_at_two() {
        let mut dp020 = blank_entity("DP-020");
        dp020.title = "Strategy".to_owned();
        dp020.r#type = "pattern".to_owned();

        let mut law042 = blank_entity("LAW-042");
        law042.title = "Open/Closed Principle".to_owned();
        law042.r#type = "law".to_owned();

        let mut dp010 = blank_entity("DP-010");
        dp010.title = "Facade".to_owned();
        dp010.r#type = "pattern".to_owned();

        let graph = build_graph_from_entities(vec![dp020, law042, dp010]);

        // "pluggable" maps to DP-020, DP-010 — only 2 entities so both should inject
        let mut results = vec![SearchResult {
            chunk_id: "c1".to_owned(),
            text: "Some".to_owned(),
            entity_id: "DP-001".to_owned(),
            entity_type: "pattern".to_owned(),
            title: "Abstract Factory".to_owned(),
            section: "Overview".to_owned(),
            metadata_json: String::new(),
            score: 0.9,
            similarity: 0.9,
            keyword_rank: None,
            semantic_rank: None,
        }];

        inject_intent_synonyms(&graph, "pluggable", &mut results, 10);

        let injected = results
            .iter()
            .filter(|r| r.chunk_id.starts_with("synonym_"))
            .count();
        assert_eq!(injected, 2, "should inject at most 2 synonym results");
    }

    #[test]
    fn inject_intent_synonyms_skips_already_present() {
        let mut dp = blank_entity("DP-020");
        dp.title = "Strategy".to_owned();
        dp.r#type = "pattern".to_owned();

        let graph = build_graph_from_entities(vec![dp]);

        let mut results = vec![SearchResult {
            chunk_id: "c1".to_owned(),
            text: "Strategy pattern".to_owned(),
            entity_id: "DP-020".to_owned(),
            entity_type: "pattern".to_owned(),
            title: "Strategy".to_owned(),
            section: "Overview".to_owned(),
            metadata_json: String::new(),
            score: 0.9,
            similarity: 0.9,
            keyword_rank: None,
            semantic_rank: None,
        }];

        inject_intent_synonyms(&graph, "flexible code", &mut results, 10);

        assert_eq!(results.len(), 1, "should not add duplicate entity");
    }

    #[test]
    fn inject_intent_synonyms_no_match_is_noop() {
        let graph = build_graph_from_entities(vec![]);

        let mut results = vec![SearchResult {
            chunk_id: "c1".to_owned(),
            text: "Some".to_owned(),
            entity_id: "DP-001".to_owned(),
            entity_type: "pattern".to_owned(),
            title: "Abstract Factory".to_owned(),
            section: "Overview".to_owned(),
            metadata_json: String::new(),
            score: 0.9,
            similarity: 0.9,
            keyword_rank: None,
            semantic_rank: None,
        }];

        inject_intent_synonyms(&graph, "something unrelated", &mut results, 10);

        assert_eq!(results.len(), 1);
    }
}
