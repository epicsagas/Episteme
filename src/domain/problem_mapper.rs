use serde::Serialize;

// ---------------------------------------------------------------------------
// Keyword maps (ported from Python problem_mapper.py)
// ---------------------------------------------------------------------------

static PROBLEM_KEYWORDS: &[(&str, &[&str])] = &[
    (
        "teams",
        &[
            "team",
            "organization",
            "communication",
            "coordination",
            "collaboration",
            "silos",
            "structure",
            "hierarchy",
            "roles",
            "responsibilities",
            "staffing",
            "hiring",
            "onboarding",
        ],
    ),
    (
        "planning",
        &[
            "timeline",
            "schedule",
            "estimate",
            "planning",
            "deadline",
            "milestone",
            "sprint",
            "backlog",
            "prioritization",
            "velocity",
            "burndown",
            "risk",
            "resource",
        ],
    ),
    (
        "architecture",
        &[
            "architecture",
            "design",
            "system",
            "microservice",
            "module",
            "component",
            "interface",
            "api",
            "abstraction",
            "coupling",
            "cohesion",
            "pattern",
            "framework",
            "infrastructure",
            "monolith",
        ],
    ),
    (
        "quality",
        &[
            "test",
            "testing",
            "quality",
            "bug",
            "defect",
            "error",
            "coverage",
            "review",
            "qa",
            "stability",
            "reliability",
            "debugging",
            "technical debt",
            "code smell",
            "refactor",
        ],
    ),
    (
        "scalability",
        &[
            "scale",
            "performance",
            "optimization",
            "throughput",
            "latency",
            "load",
            "capacity",
            "bottleneck",
            "cache",
            "database",
            "distributed",
            "concurrent",
            "parallel",
        ],
    ),
    (
        "design",
        &[
            "design",
            "principle",
            "pattern",
            "abstraction",
            "separation",
            "consistency",
            "simplicity",
            "readability",
            "clean code",
            "solid",
            "dry",
            "kiss",
        ],
    ),
    (
        "decisions",
        &[
            "decision",
            "choice",
            "alternative",
            "tradeoff",
            "trade-off",
            "versus",
            "vs",
            "evaluation",
            "comparison",
            "pros",
            "cons",
            "criteria",
            "methodology",
        ],
    ),
];

// ---------------------------------------------------------------------------
// Intent-to-entity synonym expansion (R3)
// Maps abstract intent phrases to entity IDs that should be boosted in results.
// ---------------------------------------------------------------------------

static INTENT_SYNONYMS: &[(&str, &[&str])] = &[
    ("flexible", &["DP-020", "LAW-042"]),
    ("extensible", &["DP-020", "LAW-042"]),
    ("pluggable", &["DP-020", "DP-010"]),
    ("undo", &["DP-014", "DP-017"]),
    ("redo", &["DP-014", "DP-017"]),
    ("create objects", &["DP-001", "DP-003", "DP-002"]),
    (
        "creational",
        &["DP-001", "DP-002", "DP-003", "DP-004", "DP-005"],
    ),
    ("decouple", &["DP-006", "DP-007", "DP-010", "DP-015"]),
    (
        "loose coupling",
        &["DP-006", "DP-007", "DP-010", "LAW-043"],
    ),
    (
        "tight coupling",
        &["SMELL-19", "SMELL-20", "DP-006", "LAW-043"],
    ),
    (
        "too many responsibilities",
        &["SMELL-21", "SMELL-04", "RF-010", "LAW-042"],
    ),
    ("too many parameters", &["SMELL-02", "RF-043"]),
    (
        "nested conditionals",
        &["SMELL-06", "RF-040", "RF-033", "RF-035"],
    ),
    ("breaking changes", &["LAW-042", "SMELL-15"]),
    ("fear of refactoring", &["LAW-008"]),
];

/// Look up entity IDs whose content matches the intent expressed in `query`.
///
/// The query is lowercased and scanned for known intent phrases. Single-word
/// phrases are matched against word boundaries to avoid false hits (e.g.
/// "undo" inside "fundamental"). Multi-word phrases use substring matching
/// since they are already specific enough. All matching entity IDs are
/// collected and deduplicated before being returned.
pub fn lookup_intent_synonyms(query: &str) -> Vec<String> {
    let text = query.to_lowercase();
    let words: std::collections::HashSet<&str> = text.split_whitespace().collect();
    let mut seen = std::collections::HashSet::new();
    for (phrase, entity_ids) in INTENT_SYNONYMS {
        let matched = if phrase.contains(' ') {
            text.contains(phrase)
        } else {
            words.contains(phrase)
        };
        if matched {
            for id in *entity_ids {
                seen.insert(id.to_string());
            }
        }
    }
    let mut result: Vec<String> = seen.into_iter().collect();
    result.sort();
    result
}

static ENTITY_TYPE_KEYWORDS: &[(&str, &[&str])] = &[
    (
        "pattern",
        &[
            "pattern",
            "design pattern",
            "singleton",
            "factory",
            "observer",
            "strategy",
            "adapter",
            "decorator",
            "proxy",
            "bridge",
        ],
    ),
    (
        "refactoring",
        &[
            "refactor",
            "refactoring",
            "extract",
            "inline",
            "move",
            "rename",
            "replace",
            "simplify",
            "decompose",
            "restructure",
        ],
    ),
    (
        "law",
        &[
            "law",
            "principle",
            "rule",
            "theorem",
            "effect",
            "theory",
            "conway",
            "brooks",
            "solid",
            "dry",
            "kiss",
            "yagni",
        ],
    ),
    (
        "smell",
        &[
            "smell",
            "code smell",
            "antipattern",
            "anti-pattern",
            "bad code",
            "messy",
            "spaghetti",
            "technical debt",
        ],
    ),
];

// ---------------------------------------------------------------------------
// Output types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct SearchApproach {
    pub strategy: String,
    pub category: Option<String>,
    pub categories: Vec<String>,
    pub entity_types: Vec<String>,
    pub confidence: f64,
}

// ---------------------------------------------------------------------------
// Scoring helpers
// ---------------------------------------------------------------------------

fn score(problem: &str, keyword_map: &[(&str, &[&str])]) -> Vec<(String, f64)> {
    let text = problem.to_lowercase();
    let mut results: Vec<(String, f64)> = Vec::new();

    for (key, keywords) in keyword_map {
        let matches = keywords.iter().filter(|kw| text.contains(*kw)).count();
        if matches > 0 {
            let s = matches as f64 / keywords.len().max(5) as f64;
            let s = s.min(1.0);
            results.push((key.to_string(), s));
        }
    }

    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    results
}

pub fn map_problem_to_categories(problem: &str, limit: usize) -> Vec<(String, f64)> {
    let mut scored = score(problem, PROBLEM_KEYWORDS);
    scored.truncate(limit);
    scored
}

pub fn map_problem_to_entity_types(problem: &str) -> Vec<(String, f64)> {
    score(problem, ENTITY_TYPE_KEYWORDS)
}

pub fn suggest_search_approach(problem: &str) -> SearchApproach {
    let cat_scores = map_problem_to_categories(problem, 2);
    let etype_scores = map_problem_to_entity_types(problem);
    let etypes: Vec<String> = etype_scores.iter().map(|(t, _)| t.clone()).collect();

    if !cat_scores.is_empty() && cat_scores[0].1 > 0.7 {
        return SearchApproach {
            strategy: "category".to_owned(),
            category: Some(cat_scores[0].0.clone()),
            categories: Vec::new(),
            entity_types: etypes,
            confidence: cat_scores[0].1,
        };
    }

    if !cat_scores.is_empty() && cat_scores[0].1 > 0.3 {
        return SearchApproach {
            strategy: "hybrid".to_owned(),
            category: None,
            categories: cat_scores.iter().map(|(c, _)| c.clone()).collect(),
            entity_types: etypes,
            confidence: cat_scores[0].1,
        };
    }

    SearchApproach {
        strategy: "semantic".to_owned(),
        category: None,
        categories: Vec::new(),
        entity_types: Vec::new(),
        confidence: 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_confidence_returns_category() {
        let approach = suggest_search_approach(
            "team organization communication coordination collaboration silos structure hierarchy roles responsibilities staffing hiring",
        );
        assert_eq!(approach.strategy, "category");
        assert_eq!(approach.category.unwrap(), "teams");
        assert!(approach.confidence > 0.7);
    }

    #[test]
    fn medium_confidence_returns_hybrid() {
        let approach = suggest_search_approach("team organization communication coordination");
        assert_eq!(approach.strategy, "hybrid");
        assert!(approach.confidence > 0.3);
    }

    #[test]
    fn low_confidence_returns_semantic() {
        let approach = suggest_search_approach("something totally unrelated xyz");
        assert_eq!(approach.strategy, "semantic");
        assert_eq!(approach.confidence, 0.0);
    }

    // --- Intent synonym expansion tests (R3) ---

    #[test]
    fn intent_synonyms_flexible() {
        let ids = lookup_intent_synonyms("flexible code");
        assert_eq!(ids, vec!["DP-020", "LAW-042"]);
    }

    #[test]
    fn intent_synonyms_undo_and_redo_deduplicates() {
        let ids = lookup_intent_synonyms("undo and redo");
        assert_eq!(ids, vec!["DP-014", "DP-017"]);
    }

    #[test]
    fn intent_synonyms_unrelated_returns_empty() {
        let ids = lookup_intent_synonyms("something unrelated");
        assert!(ids.is_empty());
    }

    #[test]
    fn intent_synonyms_word_boundary_no_false_match() {
        // "undo" must not match inside "fundamental"
        let ids = lookup_intent_synonyms("fundamental principle");
        assert!(ids.is_empty(), "should not match 'undo' inside 'fundamental'");
    }

    #[test]
    fn intent_synonyms_nested_conditionals() {
        let ids = lookup_intent_synonyms("nested conditionals");
        assert_eq!(ids, vec!["RF-033", "RF-035", "RF-040", "SMELL-06"]);
    }
}
