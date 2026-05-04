use serde::{Deserialize, Serialize};

/// Estimated effort level for a refactoring.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EffortLevel {
    Small,
    Medium,
    Large,
}

impl std::fmt::Display for EffortLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EffortLevel::Small => write!(f, "small"),
            EffortLevel::Medium => write!(f, "medium"),
            EffortLevel::Large => write!(f, "large"),
        }
    }
}

/// A refactoring suggestion ranked by composite priority score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringSuggestion {
    pub refactoring_id: String,
    pub title: String,
    pub priority_score: f64,
    pub effort: EffortLevel,
    pub principles_enforced: Vec<String>,
    pub description: String,
    pub metadata: SuggestionMetadata,
}

/// Breakdown of the composite score components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionMetadata {
    pub severity_score: f64,
    pub effort_score: f64,
    pub principle_score: f64,
    pub usage_score: f64,
}

/// A smell detection paired with its ranked refactoring suggestions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmellAnalysis {
    pub smell: serde_json::Value,
    pub suggestions: Vec<RefactoringSuggestion>,
}
