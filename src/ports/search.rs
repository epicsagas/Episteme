/// A single result from any search backend (semantic, keyword, or hybrid).
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub chunk_id: String,
    pub text: String,
    pub entity_id: String,
    pub entity_type: String,
    pub title: String,
    pub section: String,
    pub metadata_json: String,
    /// Raw cosine similarity (semantic) or 0.0 (keyword-only).
    pub similarity: f64,
    /// Final ranking score (cosine * boosts, or RRF score, or FTS rank).
    pub score: f64,
    /// Rank position from keyword search (used in hybrid RRF).
    pub keyword_rank: Option<usize>,
    /// Rank position from semantic search (used in hybrid RRF).
    pub semantic_rank: Option<usize>,
}

/// Trait for search index backends (SQLite FTS, vector store, etc.).
pub trait SearchIndex: Send + Sync {
    /// Search the index and return up to `limit` results, optionally filtered
    /// by `entity_type`.
    fn search(&self, query: &str, limit: usize, entity_type: Option<&str>) -> Vec<SearchResult>;
}
