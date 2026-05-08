//! Insight-specific MCP handlers for tacit knowledge operations.

use crate::domain::composite_graph::CompositeGraph;
use crate::domain::graph::KnowledgeGraph;
use crate::domain::types::{
    DuplicateCandidate, InsightLink, InsightLinkType, OverlapKind, UserEntity,
};
use crate::ports::graph::MutableGraphRepository;

/// Create a new user insight from free text, auto-detecting links to canonical entities.
///
/// Generates a TK-xxx ID, runs keyword matching against the canonical graph to
/// detect relevant entities, checks for duplicates in the user store, and persists
/// the insight via the composite graph.
pub fn add_insight(
    composite: &mut CompositeGraph,
    graph: &KnowledgeGraph,
    text: &str,
    tags: Option<Vec<String>>,
    linked_entities: Option<Vec<String>>,
    project: Option<&str>,
) -> serde_json::Value {
    if text.trim().is_empty() {
        return serde_json::json!({"error": "text must not be empty"});
    }

    // Generate next TK-xxx ID
    let id = match next_tk_id(composite.user_store()) {
        Ok(id) => id,
        Err(e) => return serde_json::json!({"error": e}),
    };

    // Auto-detect canonical entity links via keyword matching
    let auto_links = detect_canonical_links(graph, text);

    // Check for existing similar insights
    let similar = composite.user_store().search_user_entities(text, 5);
    let duplicates: Vec<serde_json::Value> = similar
        .iter()
        .map(|e| {
            serde_json::json!(DuplicateCandidate {
                insight_id: e.id.clone(),
                overlap: OverlapKind::Partial,
                note: format!("similar existing insight: {}", e.title),
            })
        })
        .collect();

    // Build suggested links from auto-detected + user-provided entities
    let mut suggested_links: Vec<serde_json::Value> = auto_links
        .iter()
        .filter(|link| !matches!(link.link_type, InsightLinkType::Auto))
        .map(|link| {
            serde_json::json!({
                "entity_id": link.entity_id,
                "score": format!("{:.2}", link.score),
                "link_type": link.link_type.to_string(),
            })
        })
        .collect();

    // Add user-specified entities as manual suggestions
    if let Some(ref entity_ids) = linked_entities {
        for eid in entity_ids {
            if graph.get_entity(eid).is_some() && !auto_links.iter().any(|l| l.entity_id == *eid) {
                suggested_links.push(serde_json::json!({
                    "entity_id": eid,
                    "score": "0.50",
                    "link_type": "manual",
                }));
            }
        }
    }

    // Build initial relations from auto-detected links
    let mut relations = std::collections::HashMap::new();
    for link in &auto_links {
        if matches!(link.link_type, InsightLinkType::Auto) {
            relations
                .entry("derives_from".to_owned())
                .or_insert_with(Vec::new)
                .push(link.entity_id.clone());
        }
    }
    if let Some(ref entity_ids) = linked_entities {
        for eid in entity_ids {
            relations
                .entry("applies_to".to_owned())
                .or_insert_with(Vec::new)
                .push(eid.clone());
        }
    }

    // Build tags
    let mut final_tags = tags.unwrap_or_default();
    if let Some(p) = project {
        final_tags.push(format!("project:{}", p));
    }

    let now = format_timestamp();
    let entity = UserEntity {
        id: id.clone(),
        title: truncate_title(text),
        content: text.to_owned(),
        author: "user".to_owned(),
        confidence: 0.5,
        evidence_count: 0,
        last_validated: String::new(),
        tags: final_tags,
        relations,
        created_at: now.clone(),
        updated_at: now,
    };

    if let Err(e) = composite.add_user_entity(entity) {
        return serde_json::json!({"error": format!("failed to add insight: {e}")});
    }

    // Compute correlations after adding
    let correlations = composite.user_store().compute_correlations(&id);

    let auto_links_json: Vec<serde_json::Value> = auto_links
        .iter()
        .map(|link| {
            serde_json::json!({
                "entity_id": link.entity_id,
                "score": format!("{:.2}", link.score),
                "link_type": link.link_type.to_string(),
            })
        })
        .collect();

    let related_insights: Vec<serde_json::Value> = correlations
        .iter()
        .take(5)
        .map(|c| {
            serde_json::json!({
                "insight_id": c.insight_id,
                "combined": format!("{:.2}", c.combined),
                "graph_proximity": format!("{:.2}", c.graph_proximity),
            })
        })
        .collect();

    serde_json::json!({
        "id": id,
        "auto_links": auto_links_json,
        "suggested_links": suggested_links,
        "related_insights": related_insights,
        "duplicates": duplicates,
        "confidence": 0.5,
    })
}

/// Confirm auto-detected links for an insight.
///
/// Creates `derives_from` relations for each accepted entity ID and bumps confidence.
pub fn confirm_links(
    composite: &mut CompositeGraph,
    insight_id: &str,
    accepted: Vec<String>,
    rejected: Vec<String>,
    merged_with: Option<&str>,
) -> serde_json::Value {
    // Verify the insight exists
    if composite.user_store().get_user_entity(insight_id).is_none() {
        return serde_json::json!({
            "error": format!("insight not found: {insight_id}")
        });
    }

    let mut confirmed_count = 0usize;
    let mut errors: Vec<String> = Vec::new();

    for entity_id in &accepted {
        match composite.add_user_relation(insight_id, "derives_from", entity_id) {
            Ok(()) => confirmed_count += 1,
            Err(e) => errors.push(format!("{}: {e}", entity_id)),
        }
    }

    // Bump confidence: +0.05 per confirmed link, capped at 1.0
    if confirmed_count > 0 {
        let bonus = 0.05 * confirmed_count as f64;
        if let Some(mut entity) = composite.user_store().get_user_entity(insight_id) {
            entity.confidence = (entity.confidence + bonus).min(1.0);
            entity.updated_at = format_timestamp();
            if let Err(e) = composite.update_user_entity(insight_id, entity) {
                errors.push(format!("confidence update failed: {e}"));
            }
        }
    }

    // Handle merge if requested
    if let Some(merge_target) = merged_with {
        if composite
            .user_store()
            .get_user_entity(merge_target)
            .is_none()
        {
            errors.push(format!("merge target not found: {merge_target}"));
        } else if let Err(e) = composite.add_user_relation(insight_id, "supersedes", merge_target) {
            errors.push(format!("merge relation failed: {e}"));
        }
    }

    if errors.is_empty() {
        serde_json::json!({
            "status": "ok",
            "insight_id": insight_id,
            "confirmed": confirmed_count,
            "rejected": rejected.len(),
            "merged_with": merged_with,
        })
    } else {
        serde_json::json!({
            "status": "partial",
            "insight_id": insight_id,
            "confirmed": confirmed_count,
            "rejected": rejected.len(),
            "merged_with": merged_with,
            "errors": errors,
        })
    }
}

/// Search user knowledge via the user store's FTS5 index.
pub fn search_insights(
    user_store: &dyn MutableGraphRepository,
    query: &str,
    limit: Option<usize>,
) -> serde_json::Value {
    if query.trim().is_empty() {
        return serde_json::json!({"results": [], "count": 0});
    }

    let effective_limit = limit.unwrap_or(10);
    let results = user_store.search_user_entities(query, effective_limit);

    let json_results: Vec<serde_json::Value> = results
        .iter()
        .map(|e| {
            let correlations = user_store.compute_correlations(&e.id);
            let related: Vec<serde_json::Value> = correlations
                .iter()
                .take(3)
                .map(|c| {
                    serde_json::json!({
                        "insight_id": c.insight_id,
                        "combined": format!("{:.2}", c.combined),
                    })
                })
                .collect();

            serde_json::json!({
                "id": e.id,
                "title": e.title,
                "content": e.content,
                "tags": e.tags,
                "confidence": e.confidence,
                "author": e.author,
                "created_at": e.created_at,
                "related": related,
            })
        })
        .collect();

    serde_json::json!({
        "results": json_results,
        "count": json_results.len(),
    })
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Compute the next TK-xxx ID by scanning existing user entity IDs.
fn next_tk_id(user_store: &dyn MutableGraphRepository) -> Result<String, String> {
    let ids = user_store.all_user_entity_ids();
    let max_num = ids
        .iter()
        .filter_map(|id| {
            id.strip_prefix("TK-")
                .and_then(|num| num.parse::<usize>().ok())
        })
        .max()
        .unwrap_or(0);

    let next = max_num + 1;
    Ok(format!("TK-{:03}", next))
}

/// Detect canonical entities that are relevant to the given text via keyword matching.
///
/// Uses the same scoring approach as `mcp_search::keyword_search`: count term matches
/// across entity text fields, with title hits weighted higher.
fn detect_canonical_links(graph: &KnowledgeGraph, text: &str) -> Vec<InsightLink> {
    let text_lower = text.to_lowercase();
    let terms: Vec<&str> = text_lower.split_whitespace().collect();

    if terms.is_empty() {
        return Vec::new();
    }

    let all_ids = graph.all_entity_ids();
    let ids_ref: Vec<&str> = all_ids.iter().map(|s| s.as_str()).collect();
    let batch = graph.get_entities_batch(&ids_ref);

    let mut scored: Vec<(String, usize)> = Vec::new();

    for (id, entity) in &batch {
        let title_lower = entity.title.to_lowercase();

        let mut text_parts = vec![
            title_lower.clone(),
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
        let searchable = text_parts.join(" ");

        let total_matches = terms
            .iter()
            .filter(|term| searchable.contains(*term))
            .count();
        if total_matches == 0 {
            continue;
        }

        let title_matches = terms
            .iter()
            .filter(|term| title_lower.contains(*term))
            .count();
        let composite_score = (total_matches << 8) | title_matches.min(255);
        scored.push((id.clone(), composite_score));
    }

    scored.sort_by_key(|b| std::cmp::Reverse(b.1));
    scored.truncate(5);

    scored
        .into_iter()
        .map(|(id, composite_score)| {
            let total = composite_score >> 8;
            let score = (total as f64 / terms.len().max(1) as f64).min(1.0);
            let link_type = if score >= 0.5 {
                InsightLinkType::Auto
            } else {
                InsightLinkType::Suggested
            };
            InsightLink {
                entity_id: id,
                score,
                link_type,
            }
        })
        .collect()
}

/// Derive a short title from the insight text (first line or first 80 chars).
fn truncate_title(text: &str) -> String {
    let first_line = text.lines().next().unwrap_or(text);
    if first_line.len() > 80 {
        format!("{}...", &first_line[..77])
    } else {
        first_line.to_owned()
    }
}

/// Produce an ISO-8601 UTC timestamp without depending on chrono.
fn format_timestamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;
    let (year, month, day) = days_to_ymd(days_since_epoch);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

/// Convert days since Unix epoch to (year, month, day).
fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    let mut year = 1970u64;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }
    let leap = is_leap(year);
    let month_days: [u64; 12] = if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1u64;
    for &md in &month_days {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }
    (year, month, days + 1)
}

fn is_leap(year: u64) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::user_graph_store::UserGraphStore;
    use crate::domain::graph::tests::{blank_entity, build_graph_from_entities};
    use crate::domain::types::Entity;

    fn build_composite(canonical_entities: Vec<Entity>) -> CompositeGraph {
        let kg = build_graph_from_entities(canonical_entities);
        let store = UserGraphStore::open_in_memory().unwrap();
        CompositeGraph::new(kg, Box::new(store))
    }

    fn make_user_entity(id: &str, title: &str) -> UserEntity {
        UserEntity {
            id: id.to_owned(),
            title: title.to_owned(),
            content: format!("Content for {title}"),
            author: "test".to_owned(),
            confidence: 0.5,
            evidence_count: 0,
            last_validated: String::new(),
            tags: vec![],
            relations: std::collections::HashMap::new(),
            created_at: "2026-01-01T00:00:00Z".to_owned(),
            updated_at: "2026-01-01T00:00:00Z".to_owned(),
        }
    }

    #[test]
    fn next_tk_id_starts_at_001() {
        let store = UserGraphStore::open_in_memory().unwrap();
        let id = next_tk_id(&store).unwrap();
        assert_eq!(id, "TK-001");
    }

    #[test]
    fn next_tk_id_increments() {
        let store = UserGraphStore::open_in_memory().unwrap();
        store
            .add_entity(make_user_entity("TK-001", "Existing"))
            .unwrap();
        let id = next_tk_id(&store).unwrap();
        assert_eq!(id, "TK-002");
    }

    #[test]
    fn add_insight_generates_id_and_auto_links() {
        let mut smell = blank_entity("SMELL-01");
        smell.title = "Long Method".to_owned();
        smell.r#type = "smell".to_owned();

        let mut composite = build_composite(vec![smell]);
        let graph = composite.canonical().clone();

        let result = add_insight(
            &mut composite,
            &graph,
            "Long method causing issues in our codebase",
            None,
            None,
            None,
        );

        assert!(result.get("error").is_none(), "should not have error");
        assert_eq!(result["id"], "TK-001");
        let auto_links = result["auto_links"].as_array().unwrap();
        assert!(!auto_links.is_empty(), "should detect SMELL-01 link");
    }

    #[test]
    fn add_insight_empty_text_returns_error() {
        let mut composite = build_composite(vec![]);
        let graph = composite.canonical().clone();

        let result = add_insight(&mut composite, &graph, "", None, None, None);
        assert!(result.get("error").is_some());
    }

    #[test]
    fn add_insight_with_project_tag() {
        let mut composite = build_composite(vec![]);
        let graph = composite.canonical().clone();

        let result = add_insight(
            &mut composite,
            &graph,
            "Some insight text",
            Some(vec!["architecture".to_owned()]),
            None,
            Some("backend"),
        );

        assert!(result.get("error").is_none());
        let entity = composite.user_store().get_user_entity("TK-001").unwrap();
        assert!(entity.tags.contains(&"architecture".to_owned()));
        assert!(entity.tags.contains(&"project:backend".to_owned()));
    }

    #[test]
    fn confirm_links_adds_relations() {
        let mut composite = build_composite(vec![]);
        let graph = composite.canonical().clone();

        // Add insight first
        let add_result = add_insight(&mut composite, &graph, "Test insight", None, None, None);
        let insight_id = add_result["id"].as_str().unwrap();

        // Add a target entity to link to
        composite
            .user_store()
            .add_entity(make_user_entity("TK-002", "Target"))
            .unwrap();

        let result = confirm_links(
            &mut composite,
            insight_id,
            vec!["TK-002".to_owned()],
            vec![],
            None,
        );
        assert_eq!(result["status"], "ok");
        assert_eq!(result["confirmed"], 1);
    }

    #[test]
    fn confirm_links_unknown_insight_returns_error() {
        let mut composite = build_composite(vec![]);

        let result = confirm_links(
            &mut composite,
            "TK-999",
            vec!["DP-001".to_owned()],
            vec![],
            None,
        );
        assert!(result.get("error").is_some());
    }

    #[test]
    fn confirm_links_bumps_confidence() {
        let mut composite = build_composite(vec![]);
        let graph = composite.canonical().clone();

        let add_result = add_insight(&mut composite, &graph, "Test insight", None, None, None);
        let insight_id = add_result["id"].as_str().unwrap();

        composite
            .user_store()
            .add_entity(make_user_entity("TK-002", "Target"))
            .unwrap();

        confirm_links(
            &mut composite,
            insight_id,
            vec!["TK-002".to_owned()],
            vec![],
            None,
        );

        let entity = composite.user_store().get_user_entity(insight_id).unwrap();
        assert!(
            (entity.confidence - 0.55).abs() < f64::EPSILON,
            "expected 0.55, got {}",
            entity.confidence
        );
    }

    #[test]
    fn search_insights_returns_empty_for_no_match() {
        let store = UserGraphStore::open_in_memory().unwrap();
        let result = search_insights(&store, "nonexistent", None);
        assert_eq!(result["count"], 0);
    }

    #[test]
    fn search_insights_empty_query_returns_empty() {
        let store = UserGraphStore::open_in_memory().unwrap();
        let result = search_insights(&store, "", None);
        assert_eq!(result["count"], 0);
    }

    #[test]
    fn search_insights_finds_matching_entities() {
        let store = UserGraphStore::open_in_memory().unwrap();
        let mut entity = make_user_entity("TK-001", "Strategy Decision");
        entity.content = "We chose Strategy pattern for payment".to_owned();
        entity.tags = vec!["decision".to_owned()];
        store.add_entity(entity).unwrap();

        let result = search_insights(&store, "Strategy payment", None);
        assert_eq!(result["count"], 1);
        let results = result["results"].as_array().unwrap();
        assert_eq!(results[0]["id"], "TK-001");
    }

    #[test]
    fn detect_canonical_links_finds_matching_entities() {
        let mut smell = blank_entity("SMELL-01");
        smell.title = "Long Method".to_owned();
        smell.r#type = "smell".to_owned();

        let dp = blank_entity("DP-001");
        let graph = build_graph_from_entities(vec![smell, dp]);

        let links = detect_canonical_links(&graph, "long method is problematic");
        assert!(!links.is_empty());
        assert_eq!(links[0].entity_id, "SMELL-01");
    }

    #[test]
    fn detect_canonical_links_empty_text() {
        let graph = build_graph_from_entities(vec![]);
        let links = detect_canonical_links(&graph, "");
        assert!(links.is_empty());
    }

    #[test]
    fn truncate_title_short_text_unchanged() {
        assert_eq!(truncate_title("Short text"), "Short text");
    }

    #[test]
    fn truncate_title_long_text_truncates() {
        let long = "a".repeat(100);
        let truncated = truncate_title(&long);
        assert_eq!(truncated.len(), 80);
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn format_timestamp_produces_iso8601() {
        let ts = format_timestamp();
        assert!(ts.ends_with('Z'));
        assert!(ts.contains('T'));
        // Should be parseable as a date
        let parts: Vec<&str> = ts.split('T').collect();
        assert_eq!(parts.len(), 2);
        let date_parts: Vec<&str> = parts[0].split('-').collect();
        assert_eq!(date_parts.len(), 3);
    }
}
