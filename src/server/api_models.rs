use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Health
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_secs: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Components>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<CacheInfo>,
}

#[derive(Debug, Serialize)]
pub struct Components {
    pub knowledge_graph: String,
    pub rag_database: String,
    pub embedding_provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_dim: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct SystemInfo {
    pub memory_used_mb: f64,
    pub memory_total_mb: f64,
    pub cpu_usage_percent: f32,
    pub disk_used_gb: f64,
    pub disk_total_gb: f64,
}

#[derive(Debug, Serialize)]
pub struct CacheInfo {
    pub enabled: bool,
    pub connected: bool,
    pub keys: u64,
    pub total_keys: u64,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
}

// ---------------------------------------------------------------------------
// Stats
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub total_entities: usize,
    pub total_edges: usize,
    pub by_type: std::collections::HashMap<String, usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<EmbeddingInfo>,
}

/// Embedding model metadata: what's stored in the DB vs what's currently configured.
#[derive(Debug, Serialize)]
pub struct EmbeddingInfo {
    pub stored_model: Option<String>,
    pub stored_dim: Option<usize>,
    pub configured_model: Option<String>,
    pub mismatch: bool,
}

// ---------------------------------------------------------------------------
// Code analysis
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct AnalyzeRequest {
    pub code: String,
    pub language: Option<String>,
    pub min_confidence: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct AnalyzeResponse {
    pub smells: Vec<serde_json::Value>,
    pub count: usize,
}

// ---------------------------------------------------------------------------
// Refactoring suggestions
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct RefactorRequest {
    pub code: String,
    pub language: Option<String>,
    pub top_k: Option<usize>,
    pub min_confidence: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct RefactorResponse {
    pub analyses: Vec<serde_json::Value>,
    pub count: usize,
}

// ---------------------------------------------------------------------------
// Search
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub limit: Option<usize>,
    pub entity_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GraphNeighborsRequest {
    pub entity_id: String,
    pub relation_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<serde_json::Value>,
    pub count: usize,
}

// ---------------------------------------------------------------------------
// Graph path
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct GraphPathRequest {
    pub from_id: String,
    pub to_id: String,
    pub max_depth: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct GraphPathResponse {
    pub from: String,
    pub to: String,
    pub length: usize,
    pub path: Vec<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// ---------------------------------------------------------------------------
// Subgraph extraction
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SubgraphRequest {
    pub entity_id: String,
    pub depth: Option<usize>,
}
