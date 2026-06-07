use regex::Regex;
use serde::Serialize;
use std::sync::LazyLock;

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
// Homonym context demotion (anti-FP)
// Maps entity IDs to non-SWE domain signals that indicate a false match.
// When the query contains any of these signals AND does NOT contain any
// SWE keyword, the entity is demoted.  The SWE-keyword guard prevents
// collateral damage on legitimate queries (e.g. "factory pattern for thread
// safety" contains "pattern" → not demoted despite "safety" signal).
// ---------------------------------------------------------------------------

/// SWE-domain keywords that, when present as whole words in the query, suppress homonym demotion.
/// Uses word-boundary matching to avoid false positives like "api" matching inside "rapid".
/// NOTE: Generic English words like "design", "engineering", "class", "code", "method",
/// "algorithm", "interface", "abstraction" are intentionally EXCLUDED because they appear
/// in non-SWE contexts (e.g. "board game design", "material engineering", "class sizes").
static SWE_GUARD_KEYWORDS: &[&str] = &[
    "software",
    "pattern",
    "patterns",
    "programming",
    "design pattern",
    "refactor",
    "refactoring",
    "function",
    "module",
    "architecture",
    "testing",
    "test",
    "api",
    "database",
    "implementation",
    "coupling",
    "cohesion",
    "smell",
    "antipattern",
    "debug",
    "deploy",
    "runtime",
    "compile",
    "source",
    "library",
    "framework",
    "developer",
    "python",
    "java",
    "rust",
    "golang",
    "typescript",
    "javascript",
    "oop",
    "solid",
    "clean code",
    "dry",
    "kiss",
    "yagni",
    "thread",
    "concurrent",
    "async",
    "callback",
    "generic",
    "trait",
    "struct",
    "enum",
    "closure",
    "dependency injection",
];

/// Compiled regex for SWE guard — matches any guard keyword as a whole word.
static SWE_GUARD_RE: LazyLock<Regex> = LazyLock::new(|| {
    let pattern = SWE_GUARD_KEYWORDS
        .iter()
        .map(|kw| format!(r"\b{}\b", regex::escape(kw)))
        .collect::<Vec<_>>()
        .join("|");
    Regex::new(&pattern).unwrap()
});

static HOMONYM_CONTEXTS: &[(&str, &[&str])] = &[
    // GoF pattern names colliding with common English
    (
        "DP-001",
        &["safety", "manufacturing", "industrial", "protocols"],
    ),
    (
        "DP-002",
        &["career", "construction", "building a", "progression"],
    ),
    (
        "DP-003",
        &["safety", "manufacturing", "industrial", "protocols"],
    ),
    (
        "DP-004",
        &["rapid", "mvp", "electric vehicle", "automotive"],
    ),
    ("DP-005", &["lambda calculus", "variables in"]),
    ("DP-006", &["hardware", "power outlet", "international"]),
    (
        "DP-007",
        &[
            "docker",
            "networking",
            "loan",
            "real estate",
            "network bridge",
        ],
    ),
    ("DP-008", &["material", "engineering", "materials science"]),
    ("DP-009", &["python decorator", "language feature"]),
    (
        "DP-010",
        &["renovation", "historical building", "building architecture"],
    ),
    ("DP-011", &["boxing", "championship", "weight class"]),
    ("DP-012", &["server", "voting", "shareholder"]),
    (
        "DP-014",
        &["line interface", "cli", "military", "hierarchy"],
    ),
    ("DP-015", &["protocol in python"]),
    ("DP-016", &["variable in statistics", "statistical"]),
    ("DP-017", &["keepsake", "photography"]),
    ("DP-018", &["bias", "user studies", "surveillance"]),
    ("DP-019", &["react", "component", "union address"]),
    (
        "DP-020",
        &[
            "board game",
            "market penetration",
            "scheduling",
            "work hours",
            "chess",
        ],
    ),
    ("DP-021", &["resume", "document"]),
    ("DP-022", &["management system", "office", "visitor"]),
    ("DP-023", &["corporate governance", "accountability"]),
    ("DP-025", &["gifts", "holiday"]),
    // Refactorings colliding with common English
    ("RF-001", &["csv", "data from"]),
    ("RF-002", &["csv", "data from"]),
    ("RF-003", &["css", "html"]),
    ("RF-004", &["css", "html"]),
    ("RF-005", &["battery", "smoke detector"]),
    ("RF-006", &["batch operation", "files"]),
    ("RF-023", &["json", "token", "jwt"]),
    ("RF-024", &["json", "token", "jwt"]),
    ("RF-026", &["gifts", "holiday"]),
    ("RF-015", &["archive", "directory"]),
    ("RF-016", &["archive", "directory"]),
    // Laws colliding with other fields
    ("LAW-001", &["physics"]),
    ("LAW-002", &["legal"]),
    ("LAW-005", &["greek mythology"]),
    ("LAW-010", &["manufacturing"]),
    ("LAW-011", &["triviality"]),
    ("LAW-014", &["economics"]),
    ("LAW-017", &["political", "powers"]),
    ("LAW-018", &["art", "modern art"]),
    ("LAW-019", &["pharmaceutical", "drug"]),
    ("LAW-020", &["mechanical", "shaft"]),
    ("LAW-021", &["chemistry", "molecular"]),
    ("LAW-022", &["estate", "family"]),
    ("LAW-023", &["biology", "evolution"]),
    ("LAW-028", &["criminology", "windows theory"]),
    ("LAW-029", &["financial"]),
    ("LAW-036", &["food", "diet", "nutrition"]),
    ("LAW-035", &["food", "diet", "nutrition"]),
    ("LAW-034", &["food", "diet", "nutrition"]),
    ("LAW-033", &["food", "diet", "nutrition"]),
    ("LAW-032", &["food", "diet", "nutrition"]),
    ("LAW-037", &["cloud", "on-premise", "infrastructure"]),
    ("LAW-039", &["animal", "movement"]),
    ("LAW-040", &["humor", "comedy", "writing essays"]),
    ("LAW-041", &["life advice"]),
    ("LAW-042", &["work hours", "scheduling"]),
    // Smells colliding with non-SWE context
    ("SMELL-01", &["pep 8", "naming"]),
    ("SMELL-04", &["school", "education", "classroom"]),
    ("SMELL-05", &["hair", "styling"]),
    ("SMELL-11", &["jvm", "class loading"]),
    ("SMELL-13", &["plagiarism"]),
    ("SMELL-15", &["family", "disputes"]),
    ("SMELL-17", &["compiler", "elimination"]),
    ("SMELL-18", &["consumer", "psychology"]),
    ("SMELL-20", &["queue", "latency"]),
];

/// Check whether the query's context conflicts with the given entity.
///
/// Returns `true` when:
/// 1. The query contains a non-SWE domain signal that matches the entity's
///    homonym context, AND
/// 2. The query does NOT contain any SWE-domain guard keywords.
///
/// This prevents demoting legitimate SWE queries that happen to share a word
/// with a non-SWE context (e.g. "factory pattern for thread safety" contains
/// "safety" but also "pattern" → not demoted).
pub fn is_homonym_mismatch(query: &str, entity_id: &str) -> bool {
    let query_lower = query.to_lowercase();

    // Check SWE guard: if query contains any SWE keyword as a whole word, never demote
    if SWE_GUARD_RE.is_match(&query_lower) {
        return false;
    }

    for (eid, signals) in HOMONYM_CONTEXTS {
        if *eid == entity_id {
            for signal in *signals {
                if query_lower.contains(signal) {
                    return true;
                }
            }
            return false;
        }
    }
    false
}

/// Compute a demotion multiplier for an entity given the query context.
///
/// Returns 0.0 when a homonym mismatch is detected (entity excluded from results),
/// 1.0 otherwise.
pub fn homonym_demotion(query: &str, entity_id: &str) -> f64 {
    if is_homonym_mismatch(query, entity_id) {
        0.0
    } else {
        1.0
    }
}

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
    ("loose coupling", &["DP-006", "DP-007", "DP-010", "LAW-043"]),
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
        assert!(
            ids.is_empty(),
            "should not match 'undo' inside 'fundamental'"
        );
    }

    #[test]
    fn intent_synonyms_nested_conditionals() {
        let ids = lookup_intent_synonyms("nested conditionals");
        assert_eq!(ids, vec!["RF-033", "RF-035", "RF-040", "SMELL-06"]);
    }

    #[test]
    fn homonym_mismatch_factory_safety() {
        assert!(is_homonym_mismatch("factory safety protocols", "DP-001"));
        assert!(is_homonym_mismatch("factory safety protocols", "DP-003"));
    }

    #[test]
    fn homonym_mismatch_strategy_market() {
        assert!(is_homonym_mismatch(
            "strategy for market penetration",
            "DP-020"
        ));
    }

    #[test]
    fn homonym_mismatch_command_interface() {
        // "interface" removed from SWE guard → mismatch detected → demoted
        assert!(is_homonym_mismatch(
            "command line interface tutorial",
            "DP-014"
        ));
    }

    #[test]
    fn swe_guard_word_boundary() {
        // "rapid" is a signal for DP-004, no SWE guard word matches → demoted
        assert!(is_homonym_mismatch(
            "prototype rapid MVP development",
            "DP-004"
        ));
        // "pattern" is a SWE guard keyword → not demoted
        assert!(!is_homonym_mismatch("factory api design pattern", "DP-001"));
    }

    #[test]
    fn homonym_demotion_returns_zero() {
        assert_eq!(homonym_demotion("factory safety protocols", "DP-001"), 0.0);
        assert_eq!(
            homonym_demotion("strategy for market penetration", "DP-020"),
            0.0
        );
        assert_eq!(homonym_demotion("factory pattern", "DP-001"), 1.0);
    }
}
