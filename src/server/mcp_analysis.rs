//! Analysis domain: code smell detection and refactoring suggestions.

use crate::adapters::constants::MAX_CODE_BYTES;
use crate::adapters::regex_parsers::get_parser;
use crate::domain::engine::RefactoringInferenceEngine;
use crate::domain::graph::KnowledgeGraph;

/// Detect code smells in source code using the parser framework.
pub fn analyze_code(code: &str, language: Option<&str>) -> serde_json::Value {
    if code.len() > MAX_CODE_BYTES {
        return serde_json::json!({
            "error": "Code input exceeds 500 KB limit."
        });
    }

    let lang = language.unwrap_or("python").to_lowercase();

    let parser = match get_parser(&lang) {
        Ok(p) => p,
        Err(e) => {
            return serde_json::json!({
                "error": e
            });
        }
    };

    let detections = parser.parse_code(code, "input");

    serde_json::json!({
        "smells": detections,
        "count": detections.len(),
    })
}

/// Detect smells and suggest ranked refactorings from the knowledge graph.
///
/// Uses RefactoringInferenceEngine for composite scoring:
/// 0.4*severity + 0.3*effort + 0.2*principle + 0.1*usage
pub fn suggest_refactorings(
    graph: &KnowledgeGraph,
    code: &str,
    language: Option<&str>,
    top_k: Option<usize>,
) -> serde_json::Value {
    if code.len() > MAX_CODE_BYTES {
        return serde_json::json!({
            "error": "Code input exceeds 500 KB limit."
        });
    }

    let lang = language.unwrap_or("python").to_lowercase();
    let top_k = top_k.unwrap_or(3).clamp(1, 10);

    let parser = match get_parser(&lang) {
        Ok(p) => p,
        Err(e) => {
            return serde_json::json!({
                "error": e
            });
        }
    };

    let detections = parser.parse_code(code, "input");

    if detections.is_empty() {
        return serde_json::json!({
            "analyses": [],
            "count": 0,
        });
    }

    let engine = RefactoringInferenceEngine::new(graph.clone());
    let analyses = engine.analyze_detections(&detections, top_k);

    let analyses_json: Vec<serde_json::Value> = analyses
        .into_iter()
        .map(|a| {
            let suggestions_json: Vec<serde_json::Value> = a
                .suggestions
                .into_iter()
                .map(|s| {
                    serde_json::json!({
                        "refactoring_id": s.refactoring_id,
                        "title": s.title,
                        "priority_score": format!("{:.3}", s.priority_score),
                        "effort": format!("{:?}", s.effort).to_lowercase(),
                        "principles_enforced": s.principles_enforced,
                        "description": s.description,
                    })
                })
                .collect();

            serde_json::json!({
                "smell": a.smell,
                "suggestions": suggestions_json,
            })
        })
        .collect();

    let count = analyses_json.len();
    serde_json::json!({
        "analyses": analyses_json,
        "count": count,
    })
}
