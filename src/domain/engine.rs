use std::collections::HashSet;

use crate::domain::graph::KnowledgeGraph;
use crate::domain::inference::{EffortLevel, RefactoringSuggestion, SuggestionMetadata};
use crate::domain::metrics::SmellDetection;
use crate::domain::types::Entity;

// ---------------------------------------------------------------------------
// RefactoringRanker — merged from syntagma-infer/src/ranker.rs
// ---------------------------------------------------------------------------

/// Ranks refactoring suggestions based on multiple criteria.
///
/// Ported from `syntagma.cli.infer.RefactoringRanker` (Python).
///
/// Composite scoring formula:
/// - Severity weight: 40% (smell confidence)
/// - Effort inverse: 30% (prefer smaller effort)
/// - Principle alignment: 20% (how many violated principles it fixes)
/// - Usage frequency: 10% (popularity heuristic)
pub struct RefactoringRanker {
    graph: KnowledgeGraph,
}

// Effort weights (inverse -- smaller effort = higher score).
const EFFORT_WEIGHT_SMALL: f64 = 1.0;
const EFFORT_WEIGHT_MEDIUM: f64 = 0.6;
const EFFORT_WEIGHT_LARGE: f64 = 0.3;

impl RefactoringRanker {
    pub fn new(graph: KnowledgeGraph) -> Self {
        Self { graph }
    }

    /// Rank refactorings based on multiple criteria.
    ///
    /// Returns suggestions sorted by priority score descending.
    pub fn rank_refactorings(
        &self,
        detection: &SmellDetection,
        refactoring_ids: &[String],
    ) -> Vec<RefactoringSuggestion> {
        let mut suggestions = Vec::new();

        for rf_id in refactoring_ids {
            let Some(rf_entity) = self.graph.get_entity(rf_id) else {
                continue;
            };

            // Calculate composite score
            let severity_score = detection.confidence; // 0.0 - 1.0
            let effort_score = self.calculate_effort_score(rf_entity);
            let principle_score = self.calculate_principle_alignment(detection, rf_entity);
            let usage_score = self.calculate_usage_frequency(rf_id);

            let priority_score = 0.4 * severity_score
                + 0.3 * effort_score
                + 0.2 * principle_score
                + 0.1 * usage_score;

            // Get effort level
            let effort = Self::extract_effort(rf_entity);

            // Get enforced principles
            let enforced = rf_entity
                .relations
                .get("enforces")
                .cloned()
                .unwrap_or_default();

            // Generate description
            let description = self.generate_description(detection, rf_entity, &enforced);

            suggestions.push(RefactoringSuggestion {
                refactoring_id: rf_id.clone(),
                title: if rf_entity.title.is_empty() {
                    "Unknown".to_owned()
                } else {
                    rf_entity.title.clone()
                },
                priority_score,
                effort,
                principles_enforced: enforced,
                description,
                metadata: SuggestionMetadata {
                    severity_score,
                    effort_score,
                    principle_score,
                    usage_score,
                },
            });
        }

        // Sort by priority descending
        suggestions.sort_by(|a, b| {
            b.priority_score
                .partial_cmp(&a.priority_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        suggestions
    }

    /// Estimate effort score (inverse of effort).
    fn calculate_effort_score(&self, rf_entity: &Entity) -> f64 {
        match Self::extract_effort(rf_entity) {
            EffortLevel::Small => EFFORT_WEIGHT_SMALL,
            EffortLevel::Medium => EFFORT_WEIGHT_MEDIUM,
            EffortLevel::Large => EFFORT_WEIGHT_LARGE,
        }
    }

    /// Extract effort estimate from entity metadata using context heuristics.
    fn extract_effort(rf_entity: &Entity) -> EffortLevel {
        let context = &rf_entity.context;

        let when_to_use = context
            .get("when_to_use")
            .map(|v| v.join(" "))
            .unwrap_or_default()
            .to_lowercase();

        let benefits = context
            .get("benefits")
            .map(|v| v.join(" "))
            .unwrap_or_default()
            .to_lowercase();

        if when_to_use.contains("simple") || benefits.contains("quick") {
            EffortLevel::Small
        } else if when_to_use.contains("complex") || benefits.contains("significant") {
            EffortLevel::Large
        } else {
            EffortLevel::Medium
        }
    }

    /// Calculate principle alignment score.
    ///
    /// How many principles violated by the smell are enforced by the refactoring?
    fn calculate_principle_alignment(&self, detection: &SmellDetection, rf_entity: &Entity) -> f64 {
        let Some(smell_entity) = self.graph.get_entity(&detection.smell_id) else {
            return 0.5;
        };

        let violated_laws: HashSet<&str> = smell_entity
            .relations
            .get("violates")
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default();

        let enforced_laws: HashSet<&str> = rf_entity
            .relations
            .get("enforces")
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default();

        if violated_laws.is_empty() {
            return 0.5; // No violation info, neutral score
        }

        let overlap = violated_laws.intersection(&enforced_laws).count();
        let max_possible = violated_laws.len();

        if max_possible > 0 {
            overlap as f64 / max_possible as f64
        } else {
            0.0
        }
    }

    /// Estimate usage frequency (popularity heuristic).
    ///
    /// More related entities = more popular. Normalized to 0.0-1.0 (max 20 relations).
    fn calculate_usage_frequency(&self, rf_id: &str) -> f64 {
        let Some(rf_entity) = self.graph.get_entity(rf_id) else {
            return 0.0;
        };

        let total_relations: usize = rf_entity.relations.values().map(|v| v.len()).sum();

        (total_relations as f64 / 20.0).min(1.0)
    }

    /// Generate human-readable description.
    fn generate_description(
        &self,
        detection: &SmellDetection,
        rf_entity: &Entity,
        enforced_laws: &[String],
    ) -> String {
        let rf_title = if rf_entity.title.is_empty() {
            "Unknown"
        } else {
            &rf_entity.title
        };

        // Get principle names (limit to 3)
        let mut law_names = Vec::new();
        for law_id in enforced_laws.iter().take(3) {
            if let Some(law_entity) = self.graph.get_entity(law_id) {
                let name = if law_entity.title.is_empty() {
                    law_id.clone()
                } else {
                    law_entity.title.clone()
                };
                law_names.push(name);
            }
        }

        let principles_text = if law_names.is_empty() {
            "code quality".to_owned()
        } else {
            law_names.join(", ")
        };

        // Extract context benefits
        let benefit_text = rf_entity
            .context
            .get("benefits")
            .and_then(|v| v.first())
            .map(|s| s.as_str())
            .unwrap_or("improve code structure");

        format!(
            "Apply {} to {}. This addresses {} and improves {}.",
            rf_title, benefit_text, detection.smell_name, principles_text
        )
    }

    /// Provide access to the underlying graph (for engine composition).
    pub fn graph(&self) -> &KnowledgeGraph {
        &self.graph
    }
}

// ---------------------------------------------------------------------------
// RefactoringInferenceEngine — merged from syntagma-infer/src/engine.rs
// ---------------------------------------------------------------------------

/// Main engine for code smell to refactoring inference.
///
/// Ported from `syntagma.cli.infer.RefactoringInferenceEngine` (Python).
///
/// For each detected smell, the engine:
/// 1. Looks up `solved_by` refactoring IDs from the knowledge graph
/// 2. Ranks them using composite scoring
/// 3. Returns the top-k suggestions
pub struct RefactoringInferenceEngine {
    ranker: RefactoringRanker,
}

impl RefactoringInferenceEngine {
    /// Create a new engine from a loaded knowledge graph.
    pub fn new(graph: KnowledgeGraph) -> Self {
        let ranker = RefactoringRanker::new(graph);
        Self { ranker }
    }

    /// Analyze a batch of smell detections and return ranked suggestions.
    ///
    /// For each detection:
    /// - Finds refactorings that solve the smell via graph traversal
    /// - Ranks them by composite score
    /// - Truncates to `top_k`
    pub fn analyze_detections(
        &self,
        detections: &[SmellDetection],
        top_k: usize,
    ) -> Vec<crate::domain::inference::SmellAnalysis> {
        let mut results = Vec::new();

        for detection in detections {
            // Step 1: Find refactorings via graph traversal
            let refactoring_ids = self.find_refactorings_for_smell(&detection.smell_id);

            // Step 2: Rank refactorings
            let suggestions = self.ranker.rank_refactorings(detection, &refactoring_ids);

            // Step 3: Truncate to top_k
            let suggestions = suggestions.into_iter().take(top_k).collect();

            // Serialize the detection as JSON value (mirrors Python's `asdict(detection)`)
            let smell_value = serde_json::to_value(detection).unwrap_or(serde_json::Value::Null);

            results.push(crate::domain::inference::SmellAnalysis {
                smell: smell_value,
                suggestions,
            });
        }

        results
    }

    /// Find refactoring IDs that solve a code smell.
    ///
    /// Looks up the smell entity's `solved_by` relations in the knowledge graph.
    fn find_refactorings_for_smell(&self, smell_id: &str) -> Vec<String> {
        self.ranker
            .graph()
            .get_neighbors(smell_id, Some("solved_by"))
    }
}

// ---------------------------------------------------------------------------
// Tests (from both engine.rs and ranker.rs)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::inference::EffortLevel;
    use crate::domain::metrics::CodeMetrics;
    use crate::domain::types::Entity;
    use std::collections::HashMap;

    fn blank_entity(id: &str) -> Entity {
        Entity {
            id: id.to_owned(),
            r#type: String::new(),
            title: String::new(),
            description: String::new(),
            name: String::new(),
            category: String::new(),
            tags: vec![],
            relations: HashMap::new(),
            context: HashMap::new(),
            file_path: String::new(),
            source: serde_json::Value::Null,
        }
    }

    fn build_graph(entities: Vec<Entity>) -> KnowledgeGraph {
        let map: HashMap<String, Entity> =
            entities.into_iter().map(|e| (e.id.clone(), e)).collect();
        KnowledgeGraph::from_entities(map)
    }

    fn make_detection(smell_id: &str, smell_name: &str, confidence: f64) -> SmellDetection {
        SmellDetection {
            smell_id: smell_id.to_owned(),
            smell_name: smell_name.to_owned(),
            confidence,
            location: "test.py:10".to_owned(),
            function_name: "test_fn".to_owned(),
            metrics: CodeMetrics::default(),
            reasons: vec!["test reason".to_owned()],
        }
    }

    // -- Ranker tests -------------------------------------------------------

    #[test]
    fn test_extract_effort_small() {
        let mut entity = blank_entity("RF-001");
        entity.title = "Extract Method".to_owned();
        entity
            .context
            .insert("when_to_use".to_owned(), vec!["simple refactor".to_owned()]);
        assert_eq!(
            RefactoringRanker::extract_effort(&entity),
            EffortLevel::Small
        );
    }

    #[test]
    fn test_extract_effort_large() {
        let mut entity = blank_entity("RF-002");
        entity.title = "Decompose Conditional".to_owned();
        entity.context.insert(
            "benefits".to_owned(),
            vec!["significant improvement".to_owned()],
        );
        assert_eq!(
            RefactoringRanker::extract_effort(&entity),
            EffortLevel::Large
        );
    }

    #[test]
    fn test_extract_effort_medium_default() {
        let entity = blank_entity("RF-003");
        assert_eq!(
            RefactoringRanker::extract_effort(&entity),
            EffortLevel::Medium
        );
    }

    #[test]
    fn test_rank_refactorings_sorts_by_priority() {
        let mut smell_entity = blank_entity("SMELL-01");
        smell_entity.title = "Long Method".to_owned();
        smell_entity.r#type = "smell".to_owned();
        smell_entity
            .relations
            .insert("violates".to_owned(), vec!["LAW-001".to_owned()]);
        smell_entity.relations.insert(
            "solved_by".to_owned(),
            vec!["RF-001".to_owned(), "RF-002".to_owned()],
        );

        // RF-001 has more relations -> higher usage score
        let mut rf1 = blank_entity("RF-001");
        rf1.title = "Extract Method".to_owned();
        rf1.r#type = "refactoring".to_owned();
        rf1.relations
            .insert("enforces".to_owned(), vec!["LAW-001".to_owned()]);
        rf1.relations
            .insert("solves".to_owned(), vec!["SMELL-01".to_owned()]);
        rf1.relations.insert(
            "related_to".to_owned(),
            (0..12).map(|i| format!("DP-{i}")).collect(),
        );
        rf1.context
            .insert("when_to_use".to_owned(), vec!["simple".to_owned()]);
        rf1.context
            .insert("benefits".to_owned(), vec!["quick win".to_owned()]);

        // RF-002 has fewer relations -> lower usage score
        let mut rf2 = blank_entity("RF-002");
        rf2.title = "Replace Method with Method Object".to_owned();
        rf2.r#type = "refactoring".to_owned();
        rf2.relations
            .insert("enforces".to_owned(), vec!["LAW-001".to_owned()]);
        rf2.relations
            .insert("solves".to_owned(), vec!["SMELL-01".to_owned()]);
        rf2.context.insert(
            "benefits".to_owned(),
            vec!["complex restructuring".to_owned()],
        );

        let graph = build_graph(vec![smell_entity, rf1, rf2]);
        let ranker = RefactoringRanker::new(graph);

        let detection = make_detection("SMELL-01", "Long Method", 0.8);
        let rf_ids = vec!["RF-001".to_owned(), "RF-002".to_owned()];

        let suggestions = ranker.rank_refactorings(&detection, &rf_ids);
        assert_eq!(suggestions.len(), 2);

        // RF-001 should be ranked first (small effort + higher usage)
        assert_eq!(suggestions[0].refactoring_id, "RF-001");
        assert!(suggestions[0].priority_score > suggestions[1].priority_score);
    }

    #[test]
    fn test_calculate_principle_alignment_overlap() {
        let mut smell_entity = blank_entity("SMELL-01");
        smell_entity.title = "Long Method".to_owned();
        smell_entity.r#type = "smell".to_owned();
        smell_entity.relations.insert(
            "violates".to_owned(),
            vec!["LAW-001".to_owned(), "LAW-002".to_owned()],
        );

        let mut rf_entity = blank_entity("RF-001");
        rf_entity.title = "Extract Method".to_owned();
        rf_entity.r#type = "refactoring".to_owned();
        // Enforces one of the two violated laws
        rf_entity
            .relations
            .insert("enforces".to_owned(), vec!["LAW-001".to_owned()]);

        let graph = build_graph(vec![smell_entity, rf_entity]);
        let ranker = RefactoringRanker::new(graph);

        let detection = make_detection("SMELL-01", "Long Method", 0.8);
        let score = ranker.calculate_principle_alignment(
            &detection,
            ranker.graph().get_entity("RF-001").unwrap(),
        );

        // 1 overlap out of 2 violated = 0.5
        assert!((score - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_principle_alignment_no_violated_laws() {
        let smell_entity = blank_entity("SMELL-01");
        let rf_entity = blank_entity("RF-001");

        let graph = build_graph(vec![smell_entity, rf_entity]);
        let ranker = RefactoringRanker::new(graph);

        let detection = make_detection("SMELL-01", "Long Method", 0.8);
        let score = ranker.calculate_principle_alignment(
            &detection,
            ranker.graph().get_entity("RF-001").unwrap(),
        );

        // No violated laws -> neutral 0.5
        assert!((score - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_usage_frequency() {
        let mut rf_entity = blank_entity("RF-001");
        rf_entity.title = "Extract Method".to_owned();
        rf_entity.r#type = "refactoring".to_owned();
        rf_entity
            .relations
            .insert("enforces".to_owned(), vec!["LAW-001".to_owned()]);
        rf_entity.relations.insert(
            "solves".to_owned(),
            vec!["SMELL-01".to_owned(), "SMELL-02".to_owned()],
        );
        // total = 3 relations

        let graph = build_graph(vec![rf_entity]);
        let ranker = RefactoringRanker::new(graph);

        let score = ranker.calculate_usage_frequency("RF-001");
        // 3 / 20.0 = 0.15
        assert!((score - 0.15).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_usage_frequency_capped_at_one() {
        let mut rf_entity = blank_entity("RF-001");
        rf_entity.title = "Popular Refactoring".to_owned();
        rf_entity.r#type = "refactoring".to_owned();
        // 25 relations -> should cap at 1.0
        rf_entity.relations.insert(
            "enforces".to_owned(),
            (0..25).map(|i| format!("LAW-{i}")).collect(),
        );

        let graph = build_graph(vec![rf_entity]);
        let ranker = RefactoringRanker::new(graph);

        let score = ranker.calculate_usage_frequency("RF-001");
        assert!((score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_generate_description_with_laws() {
        let mut law_entity = blank_entity("LAW-001");
        law_entity.title = "Single Responsibility Principle".to_owned();
        law_entity.r#type = "law".to_owned();

        let mut rf_entity = blank_entity("RF-001");
        rf_entity.title = "Extract Method".to_owned();
        rf_entity.r#type = "refactoring".to_owned();
        rf_entity
            .relations
            .insert("enforces".to_owned(), vec!["LAW-001".to_owned()]);
        rf_entity
            .context
            .insert("benefits".to_owned(), vec!["reduce complexity".to_owned()]);

        let graph = build_graph(vec![law_entity, rf_entity]);
        let ranker = RefactoringRanker::new(graph);

        let detection = make_detection("SMELL-01", "Long Method", 0.8);
        let desc = ranker.generate_description(
            &detection,
            ranker.graph().get_entity("RF-001").unwrap(),
            &["LAW-001".to_owned()],
        );

        assert!(desc.contains("Extract Method"));
        assert!(desc.contains("reduce complexity"));
        assert!(desc.contains("Long Method"));
        assert!(desc.contains("Single Responsibility Principle"));
    }

    #[test]
    fn test_generate_description_no_laws() {
        let rf_entity = blank_entity("RF-001");

        let graph = build_graph(vec![rf_entity]);
        let ranker = RefactoringRanker::new(graph);

        let detection = make_detection("SMELL-01", "Long Method", 0.8);
        let desc = ranker.generate_description(
            &detection,
            ranker.graph().get_entity("RF-001").unwrap(),
            &[],
        );

        assert!(desc.contains("code quality"));
    }

    #[test]
    fn test_unknown_refactoring_id_skipped() {
        let graph = build_graph(vec![]);
        let ranker = RefactoringRanker::new(graph);

        let detection = make_detection("SMELL-01", "Long Method", 0.8);
        let suggestions = ranker.rank_refactorings(&detection, &["RF-NONEXISTENT".to_owned()]);

        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_composite_score_formula() {
        // Verify exact formula: 0.4 * severity + 0.3 * effort + 0.2 * principle + 0.1 * usage
        let mut smell_entity = blank_entity("SMELL-01");
        smell_entity
            .relations
            .insert("violates".to_owned(), vec!["LAW-001".to_owned()]);

        let mut rf_entity = blank_entity("RF-001");
        rf_entity.title = "Test Refactoring".to_owned();
        rf_entity.r#type = "refactoring".to_owned();
        rf_entity
            .relations
            .insert("enforces".to_owned(), vec!["LAW-001".to_owned()]);
        // 2 relations total -> usage = 2/20 = 0.1
        rf_entity
            .relations
            .insert("solves".to_owned(), vec!["SMELL-01".to_owned()]);
        // no context -> medium effort -> 0.6

        let graph = build_graph(vec![smell_entity, rf_entity]);
        let ranker = RefactoringRanker::new(graph);

        let detection = make_detection("SMELL-01", "Long Method", 0.9);
        let suggestions = ranker.rank_refactorings(&detection, &["RF-001".to_owned()]);

        assert_eq!(suggestions.len(), 1);
        let s = &suggestions[0];

        // severity = 0.9, effort = 0.6, principle = 1.0 (1/1 overlap), usage = 0.1
        let expected = 0.4 * 0.9 + 0.3 * 0.6 + 0.2 * 1.0 + 0.1 * 0.1;
        assert!(
            (s.priority_score - expected).abs() < 1e-10,
            "expected {}, got {}",
            expected,
            s.priority_score
        );

        assert!((s.metadata.severity_score - 0.9).abs() < f64::EPSILON);
        assert!((s.metadata.effort_score - 0.6).abs() < f64::EPSILON);
        assert!((s.metadata.principle_score - 1.0).abs() < f64::EPSILON);
        assert!((s.metadata.usage_score - 0.1).abs() < f64::EPSILON);
    }

    // -- Engine tests -------------------------------------------------------

    #[test]
    fn test_analyze_detections_basic() {
        let mut law = blank_entity("LAW-001");
        law.title = "Single Responsibility Principle".to_owned();
        law.r#type = "law".to_owned();

        let mut smell = blank_entity("SMELL-01");
        smell.title = "Long Method".to_owned();
        smell.r#type = "smell".to_owned();
        smell
            .relations
            .insert("violates".to_owned(), vec!["LAW-001".to_owned()]);
        smell.relations.insert(
            "solved_by".to_owned(),
            vec!["RF-001".to_owned(), "RF-002".to_owned()],
        );

        let mut rf1 = blank_entity("RF-001");
        rf1.title = "Extract Method".to_owned();
        rf1.r#type = "refactoring".to_owned();
        rf1.relations
            .insert("enforces".to_owned(), vec!["LAW-001".to_owned()]);
        rf1.relations
            .insert("solves".to_owned(), vec!["SMELL-01".to_owned()]);
        rf1.context
            .insert("when_to_use".to_owned(), vec!["simple refactor".to_owned()]);
        rf1.context
            .insert("benefits".to_owned(), vec!["quick win".to_owned()]);

        let mut rf2 = blank_entity("RF-002");
        rf2.title = "Replace Temp with Query".to_owned();
        rf2.r#type = "refactoring".to_owned();
        rf2.relations
            .insert("enforces".to_owned(), vec!["LAW-001".to_owned()]);
        rf2.relations
            .insert("solves".to_owned(), vec!["SMELL-01".to_owned()]);

        let graph = build_graph(vec![law, smell, rf1, rf2]);
        let engine = RefactoringInferenceEngine::new(graph);

        let detection = make_detection("SMELL-01", "Long Method", 0.8);
        let results = engine.analyze_detections(&[detection], 3);

        assert_eq!(results.len(), 1);
        let analysis = &results[0];

        // Smell should be serialized as JSON object
        assert!(analysis.smell.is_object());
        assert_eq!(analysis.smell["smell_id"], "SMELL-01");

        // Should have 2 suggestions (both refactorings are in the graph)
        assert_eq!(analysis.suggestions.len(), 2);

        // First should be Extract Method (small effort -> higher score)
        assert_eq!(analysis.suggestions[0].refactoring_id, "RF-001");
        assert_eq!(analysis.suggestions[0].title, "Extract Method");
    }

    #[test]
    fn test_analyze_detections_top_k() {
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
        rf2.title = "Replace Method".to_owned();
        rf2.r#type = "refactoring".to_owned();

        let mut rf3 = blank_entity("RF-003");
        rf3.title = "Decompose Conditional".to_owned();
        rf3.r#type = "refactoring".to_owned();

        let graph = build_graph(vec![smell, rf1, rf2, rf3]);
        let engine = RefactoringInferenceEngine::new(graph);

        let detection = make_detection("SMELL-01", "Long Method", 0.8);
        let results = engine.analyze_detections(&[detection], 2);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].suggestions.len(), 2);
    }

    #[test]
    fn test_analyze_detections_no_refactorings() {
        let mut smell = blank_entity("SMELL-99");
        smell.title = "Unknown Smell".to_owned();
        smell.r#type = "smell".to_owned();

        let graph = build_graph(vec![smell]);
        let engine = RefactoringInferenceEngine::new(graph);

        let detection = make_detection("SMELL-99", "Unknown Smell", 0.5);
        let results = engine.analyze_detections(&[detection], 3);

        assert_eq!(results.len(), 1);
        assert!(results[0].suggestions.is_empty());
    }

    #[test]
    fn test_analyze_multiple_detections() {
        let mut smell1 = blank_entity("SMELL-01");
        smell1.title = "Long Method".to_owned();
        smell1.r#type = "smell".to_owned();
        smell1
            .relations
            .insert("solved_by".to_owned(), vec!["RF-001".to_owned()]);

        let mut smell2 = blank_entity("SMELL-02");
        smell2.title = "Long Parameter List".to_owned();
        smell2.r#type = "smell".to_owned();
        smell2
            .relations
            .insert("solved_by".to_owned(), vec!["RF-002".to_owned()]);

        let mut rf1 = blank_entity("RF-001");
        rf1.title = "Extract Method".to_owned();
        rf1.r#type = "refactoring".to_owned();

        let mut rf2 = blank_entity("RF-002");
        rf2.title = "Introduce Parameter Object".to_owned();
        rf2.r#type = "refactoring".to_owned();

        let graph = build_graph(vec![smell1, smell2, rf1, rf2]);
        let engine = RefactoringInferenceEngine::new(graph);

        let det1 = make_detection("SMELL-01", "Long Method", 0.8);
        let det2 = make_detection("SMELL-02", "Long Parameter List", 0.7);

        let results = engine.analyze_detections(&[det1, det2], 3);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].suggestions.len(), 1);
        assert_eq!(results[0].suggestions[0].refactoring_id, "RF-001");
        assert_eq!(results[1].suggestions[0].refactoring_id, "RF-002");
    }

    #[test]
    fn test_find_refactorings_for_smell() {
        let mut smell = blank_entity("SMELL-01");
        smell.title = "Long Method".to_owned();
        smell.r#type = "smell".to_owned();
        smell.relations.insert(
            "solved_by".to_owned(),
            vec!["RF-001".to_owned(), "RF-002".to_owned()],
        );
        smell
            .relations
            .insert("violates".to_owned(), vec!["LAW-001".to_owned()]);

        let graph = build_graph(vec![smell]);
        let engine = RefactoringInferenceEngine::new(graph);

        let ids = engine.find_refactorings_for_smell("SMELL-01");
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"RF-001".to_owned()));
        assert!(ids.contains(&"RF-002".to_owned()));
    }

    #[test]
    fn test_find_refactorings_unknown_smell() {
        let graph = build_graph(vec![]);
        let engine = RefactoringInferenceEngine::new(graph);

        let ids = engine.find_refactorings_for_smell("SMELL-NONEXISTENT");
        assert!(ids.is_empty());
    }
}
