use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DetailLevel {
    Minimal,
    Summary,
    Detailed,
    Full,
}

pub fn estimate_tokens(text: &str) -> usize {
    (text.len() / 4).max(1)
}

fn build_minimal(entity: &serde_json::Value) -> serde_json::Value {
    let meta = entity.get("metadata").cloned().unwrap_or(serde_json::Value::Null);
    serde_json::json!({
        "id": entity.get("id")
            .or_else(|| entity.get("entity_id"))
            .or_else(|| meta.get("entity_id"))
            .unwrap_or(&serde_json::Value::String(String::new())),
        "title": entity.get("title").unwrap_or(&serde_json::Value::String(String::new())),
        "type": entity.get("type")
            .or_else(|| entity.get("entity_type"))
            .or_else(|| meta.get("type"))
            .unwrap_or(&serde_json::Value::String(String::new())),
        "category": entity.get("category")
            .or_else(|| meta.get("category"))
            .unwrap_or(&serde_json::Value::String(String::new())),
    })
}

fn build_summary(entity: &serde_json::Value) -> serde_json::Value {
    let mut result = build_minimal(entity);
    let ctx = entity
        .get("context")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();

    let mut parts: Vec<String> = Vec::new();
    for key in &["benefits", "when_to_use", "drawbacks"] {
        if let Some(items) = ctx.get(*key).and_then(|v| v.as_array())
            && let Some(first) = items.first()
                && let Some(s) = first.as_str() {
                    parts.push(format!("{}: {}", key, s));
                }
    }
    result["summary"] = serde_json::Value::String(parts.join("; "));
    result
}

fn truncate_context(
    ctx: &serde_json::Value,
    limit: usize,
) -> serde_json::Value {
    let map = ctx.as_object().cloned().unwrap_or_default();
    let truncated: HashMap<String, serde_json::Value> = map
        .into_iter()
        .map(|(k, v)| {
            let truncated_val = if let Some(arr) = v.as_array() {
                serde_json::Value::Array(arr.iter().take(limit).cloned().collect())
            } else {
                v
            };
            (k, truncated_val)
        })
        .collect();
    serde_json::Value::Object(
        truncated
            .into_iter()
            .collect(),
    )
}

pub fn summarize_entity(
    entity: &serde_json::Value,
    detail_level: DetailLevel,
) -> serde_json::Value {
    match detail_level {
        DetailLevel::Minimal => build_minimal(entity),
        DetailLevel::Summary => build_summary(entity),
        DetailLevel::Detailed => {
            let mut result = build_summary(entity);
            let ctx = entity.get("context").cloned().unwrap_or(serde_json::Value::Null);
            result["context"] = truncate_context(&ctx, 2);
            let relations = entity
                .get("relations")
                .and_then(|v| v.as_object())
                .cloned()
                .unwrap_or_default();
            let counts: HashMap<String, usize> = relations
                .iter()
                .map(|(k, v)| (k.clone(), v.as_array().map(|a| a.len()).unwrap_or(0)))
                .collect();
            result["relation_counts"] = serde_json::to_value(counts).unwrap_or(serde_json::Value::Null);
            result
        }
        DetailLevel::Full => entity.clone(),
    }
}

const OVERHEAD_PER_ENTITY: usize = 20;

fn estimate_entity_tokens(entity: &serde_json::Value) -> usize {
    let json_str = serde_json::to_string(entity).unwrap_or_default();
    estimate_tokens(&json_str) + OVERHEAD_PER_ENTITY
}

pub fn optimize_response(
    entities: &[serde_json::Value],
    max_tokens: usize,
) -> (Vec<serde_json::Value>, usize) {
    let mut result: Vec<serde_json::Value> = Vec::new();
    let mut tokens_used: usize = 0;

    for entity in entities {
        let remaining = max_tokens.saturating_sub(tokens_used);

        let level = if remaining < 50 {
            break;
        } else if remaining >= 300 {
            DetailLevel::Full
        } else if remaining >= 200 {
            DetailLevel::Detailed
        } else if remaining >= 100 {
            DetailLevel::Summary
        } else {
            DetailLevel::Minimal
        };

        let summarized = summarize_entity(entity, level);
        let cost = estimate_entity_tokens(&summarized);

        result.push(summarized);
        tokens_used += cost;
    }

    (result, tokens_used)
}

const DEPTH_WORDS: &[&str] = &["explain", "detail", "how", "why", "example", "implement"];

pub fn select_detail_level(query: Option<&str>, token_budget: usize) -> DetailLevel {
    if token_budget < 100 {
        return DetailLevel::Minimal;
    }
    if token_budget >= 300 {
        return DetailLevel::Full;
    }
    if let Some(q) = query {
        let lowered = q.to_lowercase();
        if DEPTH_WORDS.iter().any(|w| lowered.contains(w)) {
            return DetailLevel::Detailed;
        }
    }
    if token_budget >= 150 {
        DetailLevel::Summary
    } else {
        DetailLevel::Minimal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_tokens() {
        assert!(estimate_tokens("") >= 1);
        assert!(estimate_tokens("hello world") >= 2);
    }

    #[test]
    fn test_minimal_level() {
        let entity = serde_json::json!({
            "id": "DP-005",
            "title": "Singleton",
            "type": "pattern",
            "category": "creational"
        });
        let result = summarize_entity(&entity, DetailLevel::Minimal);
        assert_eq!(result["id"], "DP-005");
        assert_eq!(result["title"], "Singleton");
    }

    #[test]
    fn test_select_detail_level_depth_words() {
        assert_eq!(
            select_detail_level(Some("explain singleton"), 200),
            DetailLevel::Detailed
        );
        assert_eq!(
            select_detail_level(Some("what is singleton"), 200),
            DetailLevel::Summary
        );
    }

    #[test]
    fn test_select_detail_level_budget() {
        assert_eq!(select_detail_level(None, 50), DetailLevel::Minimal);
        assert_eq!(select_detail_level(None, 300), DetailLevel::Full);
    }

    #[test]
    fn test_optimize_response() {
        let entities: Vec<serde_json::Value> = (0..5)
            .map(|i| {
                serde_json::json!({
                    "id": format!("DP-{:03}", i),
                    "title": format!("Pattern {}", i),
                    "type": "pattern",
                })
            })
            .collect();
        let (result, tokens) = optimize_response(&entities, 500);
        assert!(!result.is_empty());
        assert!(tokens <= 600);
    }
}
