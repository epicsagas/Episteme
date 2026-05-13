use serde::{Deserialize, Serialize};

/// Code metrics for a function or class.
///
/// Ported from `episteme.parsers.base.CodeMetrics`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMetrics {
    pub loc: usize,
    pub cyclomatic_complexity: usize,
    pub nesting_depth: usize,
    pub parameter_count: usize,
    pub local_variables: usize,
    pub return_statements: usize,
    #[serde(default)]
    pub method_count: usize,
    #[serde(default)]
    pub field_count: usize,
    #[serde(default)]
    pub external_calls: usize,
    #[serde(default)]
    pub primitive_params: usize,
    #[serde(default)]
    pub branch_count: usize,
    #[serde(default)]
    pub method_call_chains: usize,
    #[serde(default)]
    pub delegation_methods: usize,
    #[serde(default)]
    pub ast_hash: String,
    /// Number of comment lines in the function/class body.
    #[serde(default)]
    pub comment_count: usize,
    /// Number of methods that override parent methods with empty or trivial bodies.
    #[serde(default)]
    pub override_count: usize,
}

impl Default for CodeMetrics {
    fn default() -> Self {
        Self {
            loc: 0,
            cyclomatic_complexity: 1, // base complexity
            nesting_depth: 0,
            parameter_count: 0,
            local_variables: 0,
            return_statements: 0,
            method_count: 0,
            field_count: 0,
            external_calls: 0,
            primitive_params: 0,
            branch_count: 0,
            method_call_chains: 0,
            delegation_methods: 0,
            ast_hash: String::new(),
            comment_count: 0,
            override_count: 0,
        }
    }
}

/// A detected code smell with confidence score and location.
///
/// Ported from `episteme.parsers.base.SmellDetection`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmellDetection {
    pub smell_id: String,
    pub smell_name: String,
    pub confidence: f64,
    pub location: String,
    pub function_name: String,
    pub metrics: CodeMetrics,
    pub reasons: Vec<String>,
}
