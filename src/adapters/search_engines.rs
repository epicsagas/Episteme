use std::collections::HashMap;

use rusqlite::{Connection, params};

use crate::adapters::error::{InfraError, Result};
use crate::ports::embeddings::EmbeddingProvider;

// ---------------------------------------------------------------------------
// SearchResult — internal rich result used across search, keyword, and hybrid
// ---------------------------------------------------------------------------

/// A single result from any search backend (semantic, keyword, or hybrid).
/// This is the internal representation with all ranking metadata.
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

// ---------------------------------------------------------------------------
// Re-ranking weights (ported from Python build_v2.py)
// ---------------------------------------------------------------------------

/// Sections that carry the core meaning of an entity get a boost.
/// Sections like "Implementation Details" or "Examples" are more peripheral
/// and receive no bonus (1.0 multiplier).
static SECTION_BOOST: &[(&str, f64)] = &[
    // Universal high-signal sections
    ("intent", 1.15),
    ("overview", 1.12),
    ("when to use", 1.10),
    ("motivation", 1.08),
    ("definition", 1.08),
    ("summary", 1.06),
    ("description", 1.05),
    // Pattern-specific
    ("applicability", 1.08),
    ("structure", 1.04),
    // Law/principle-specific
    ("implications", 1.06),
    ("origin", 1.03),
];

/// Query keywords that signal the user wants a specific entity type.
static TYPE_QUERY_SIGNALS: &[(&str, &[&str])] = &[
    (
        "pattern",
        &[
            "pattern",
            "design pattern",
            "gof",
            "factory",
            "singleton",
            "observer",
            "strategy",
            "decorator",
            "adapter",
            "facade",
            "proxy",
            "composite",
            "bridge",
            "flyweight",
            "template",
            "iterator",
            "mediator",
            "memento",
            "visitor",
            "command",
            "chain",
            "state",
            "abstract factory",
            "builder",
            "prototype",
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
            "split",
            "encapsulate",
            "decompose",
            "consolidate",
            "pull up",
            "push down",
        ],
    ),
    (
        "smell",
        &[
            "smell",
            "code smell",
            "anti-pattern",
            "antipattern",
            "bloat",
            "long method",
            "large class",
            "duplicate",
            "coupling",
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
            "bias",
            "conway",
            "brooks",
            "solid",
            "dry",
            "kiss",
            "yagni",
            "cap",
            "amdahl",
            "dunning",
            "occam",
            "pareto",
        ],
    ),
];

const TYPE_SIGNAL_BOOST: f64 = 1.05;

/// Return boost multiplier for the given section name.
pub fn section_boost(section: &str) -> f64 {
    let key = section.trim().to_lowercase();

    // Exact match first.
    for (pattern, boost) in SECTION_BOOST {
        if *pattern == key {
            return *boost;
        }
    }
    // Prefix match for compound section names.
    for (pattern, boost) in SECTION_BOOST {
        if key.starts_with(pattern) {
            return *boost;
        }
    }
    1.0
}

/// Return boost multiplier when the query signals a specific entity type.
pub fn type_boost(entity_type: &str, query_lower: &str) -> f64 {
    for (etype, signals) in TYPE_QUERY_SIGNALS {
        if *etype == entity_type && signals.iter().any(|sig| query_lower.contains(sig)) {
            return TYPE_SIGNAL_BOOST;
        }
    }
    1.0
}

// ---------------------------------------------------------------------------
// Cosine similarity
// ---------------------------------------------------------------------------

/// Compute cosine similarity between two f32 vectors.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| (*x as f64) * (*y as f64)).sum();
    let norm_a: f64 = a.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}

// ---------------------------------------------------------------------------
// Semantic search
// ---------------------------------------------------------------------------

/// Perform semantic search using cosine similarity with section- and
/// type-aware re-ranking.
///
/// `query_embedding` is the embedding of the search query (f32 vector).
/// `top_k` limits the number of returned results.
/// `filters` may contain `"entity_type"` and/or `"entity_id"`.
pub fn semantic_search(
    conn: &Connection,
    query_embedding: &[f32],
    top_k: usize,
    entity_type_filter: Option<&str>,
    entity_id_filter: Option<&str>,
    query_text: &str,
) -> Result<Vec<SearchResult>> {
    let query_lower = query_text.to_lowercase();

    // Build the SQL query with optional filters.
    let mut where_clauses = Vec::new();
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(et) = entity_type_filter {
        where_clauses.push("c.entity_type = ?".to_owned());
        param_values.push(Box::new(et.to_owned()));
    }
    if let Some(eid) = entity_id_filter {
        where_clauses.push("c.entity_id = ?".to_owned());
        param_values.push(Box::new(eid.to_owned()));
    }

    let where_clause = if where_clauses.is_empty() {
        String::new()
    } else {
        format!(" AND {}", where_clauses.join(" AND "))
    };

    let sql = format!(
        "SELECT c.id, c.text, c.entity_id, c.entity_type, c.title, c.section, c.metadata, e.embedding
         FROM chunks c
         JOIN embeddings e ON c.id = e.chunk_id
         WHERE 1=1{where_clause}"
    );

    let params_refs: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|x| x.as_ref()).collect();

    let mut stmt = conn.prepare(&sql).map_err(|e| InfraError::Database(e.to_string()))?;

    let rows = stmt
        .query_map(params_refs.as_slice(), |row| {
            let chunk_id: String = row.get(0)?;
            let text: String = row.get(1)?;
            let entity_id: String = row.get(2)?;
            let entity_type: String = row.get(3)?;
            let title: String = row.get(4)?;
            let section: String = row.get(5)?;
            let metadata_json: String = row.get(6)?;
            let embedding_blob: Vec<u8> = row.get(7)?;
            Ok((
                chunk_id,
                text,
                entity_id,
                entity_type,
                title,
                section,
                metadata_json,
                embedding_blob,
            ))
        })
        .map_err(|e| InfraError::Database(e.to_string()))?;

    let mut results: Vec<SearchResult> = Vec::new();

    for row in rows {
        let (chunk_id, text, entity_id, entity_type, title, section, metadata_json, embedding_blob) =
            row.map_err(|e| InfraError::Database(e.to_string()))?;

        let embedding: Vec<f32> = embedding_blob
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        let similarity = cosine_similarity(query_embedding, &embedding);
        let sec_boost = section_boost(&section);
        let t_boost = type_boost(&entity_type, &query_lower);
        let score = similarity * sec_boost * t_boost;

        results.push(SearchResult {
            chunk_id,
            text,
            entity_id,
            entity_type,
            title,
            section,
            metadata_json,
            similarity,
            score,
            keyword_rank: None,
            semantic_rank: None,
        });
    }

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(top_k);
    Ok(results)
}

// ---------------------------------------------------------------------------
// FTS5 index
// ---------------------------------------------------------------------------

/// Drop any existing FTS5 index and rebuild it from the `chunks` table.
pub fn build_fts_index(conn: &Connection) -> Result<()> {
    conn.execute_batch("DROP TABLE IF EXISTS chunks_fts")
        .map_err(|e| InfraError::Database(e.to_string()))?;

    conn.execute_batch(
        "
        CREATE VIRTUAL TABLE chunks_fts USING fts5(
            text,
            title,
            section,
            content='chunks',
            content_rowid='rowid'
        );
        ",
    )
    .map_err(|e| InfraError::Database(e.to_string()))?;

    conn.execute_batch(
        "
        INSERT INTO chunks_fts(rowid, text, title, section)
        SELECT rowid, text, COALESCE(title, ''), COALESCE(section, '')
        FROM chunks;
        ",
    )
    .map_err(|e| InfraError::Database(e.to_string()))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Query sanitization
// ---------------------------------------------------------------------------

/// Strip characters that are special to FTS5 query syntax, then wrap each
/// remaining token in double-quotes so FTS5 treats them as literal phrases.
pub fn sanitize_fts_query(query: &str) -> String {
    // Remove quotes and asterisks.
    let cleaned: String = query
        .chars()
        .map(|c| {
            if c == '"' || c == '\'' || c == '*' || (!c.is_alphanumeric() && !c.is_whitespace()) {
                ' '
            } else {
                c
            }
        })
        .collect();

    let tokens: Vec<&str> = cleaned.split_whitespace().collect();
    tokens
        .iter()
        .map(|t| format!("\"{t}\""))
        .collect::<Vec<_>>()
        .join(" ")
}

// ---------------------------------------------------------------------------
// Keyword search
// ---------------------------------------------------------------------------

/// Full-text search against the `chunks_fts` virtual table.
///
/// Returns results ordered by FTS5 rank (BM25).  When `entity_type_filter`
/// is `Some`, results are further filtered to that entity type.
pub fn keyword_search(
    conn: &Connection,
    query: &str,
    limit: usize,
    entity_type_filter: Option<&str>,
) -> Result<Vec<SearchResult>> {
    let fts_query = sanitize_fts_query(query);

    let mut results = Vec::new();

    if let Some(etype) = entity_type_filter {
        let mut stmt = conn
            .prepare(
                "
                SELECT
                    c.id,
                    c.text,
                    c.entity_id,
                    c.entity_type,
                    c.title,
                    c.section,
                    c.metadata,
                    rank AS relevance_score
                FROM chunks_fts f
                JOIN chunks c ON c.rowid = f.rowid
                WHERE chunks_fts MATCH ?1 AND c.entity_type = ?2
                ORDER BY rank
                LIMIT ?3
                ",
            )
            .map_err(|e| InfraError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(params![fts_query, etype, limit as i64], |row| {
                read_search_row(row)
            })
            .map_err(|e| InfraError::Database(e.to_string()))?;

        for row in rows {
            results.push(row.map_err(|e| InfraError::Database(e.to_string()))?);
        }
    } else {
        let mut stmt = conn
            .prepare(
                "
                SELECT
                    c.id,
                    c.text,
                    c.entity_id,
                    c.entity_type,
                    c.title,
                    c.section,
                    c.metadata,
                    rank AS relevance_score
                FROM chunks_fts f
                JOIN chunks c ON c.rowid = f.rowid
                WHERE chunks_fts MATCH ?1
                ORDER BY rank
                LIMIT ?2
                ",
            )
            .map_err(|e| InfraError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(params![fts_query, limit as i64], |row| {
                read_search_row(row)
            })
            .map_err(|e| InfraError::Database(e.to_string()))?;

        for row in rows {
            results.push(row.map_err(|e| InfraError::Database(e.to_string()))?);
        }
    }

    Ok(results)
}

/// Helper: map a single row from the keyword-search query into a
/// `SearchResult`.  The `relevance_score` is stored in `score`.
fn read_search_row(row: &rusqlite::Row<'_>) -> std::result::Result<SearchResult, rusqlite::Error> {
    let chunk_id: String = row.get(0)?;
    let text: String = row.get(1)?;
    let entity_id: String = row.get(2)?;
    let entity_type: String = row.get(3)?;
    let title: String = row.get(4)?;
    let section: String = row.get(5)?;
    let metadata_json: String = row.get(6)?;
    let relevance_score: f64 = row.get(7)?;

    Ok(SearchResult {
        chunk_id,
        text,
        entity_id,
        entity_type,
        title,
        section,
        metadata_json,
        similarity: 0.0,
        score: relevance_score,
        keyword_rank: None,
        semantic_rank: None,
    })
}

// ---------------------------------------------------------------------------
// RRF constants (ported from Python hybrid.py)
// ---------------------------------------------------------------------------

const RRF_K: usize = 20;
const KEYWORD_WEIGHT: f64 = 0.4;
const SEMANTIC_WEIGHT: f64 = 0.6;

// ---------------------------------------------------------------------------
// Hybrid search
// ---------------------------------------------------------------------------

/// Perform hybrid search combining FTS5 keyword search and semantic
/// embedding search using Reciprocal Rank Fusion (RRF).
///
/// RRF formula per chunk:  score = keyword_weight / (K + keyword_rank)
///                              + semantic_weight / (K + semantic_rank)
///
/// Graceful degradation: if one search fails, the other is used alone.
pub fn hybrid_search(
    conn: &Connection,
    provider: &dyn EmbeddingProvider,
    query: &str,
    limit: usize,
    entity_type_filter: Option<&str>,
    entity_id_filter: Option<&str>,
) -> Result<Vec<SearchResult>> {
    let expanded_limit = limit * 2;

    // --- keyword search (graceful degradation) ---
    let keyword_results: Vec<SearchResult> = keyword_search(
        conn,
        query,
        expanded_limit,
        entity_type_filter,
    ).unwrap_or_default();

    // --- semantic search (graceful degradation) ---
    let semantic_results: Vec<SearchResult> = {
        match provider.embed(query) {
            Ok(query_embedding) => {
                semantic_search(
                    conn,
                    &query_embedding,
                    expanded_limit,
                    entity_type_filter,
                    entity_id_filter,
                    query,
                ).unwrap_or_default()
            }
            Err(_) => Vec::new(),
        }
    };

    // Both failed — return empty.
    if keyword_results.is_empty() && semantic_results.is_empty() {
        return Ok(Vec::new());
    }

    // Only keyword available.
    if semantic_results.is_empty() {
        let mut results = keyword_results;
        for r in &mut results {
            r.score = r.score.abs();
        }
        results.truncate(limit);
        return Ok(results);
    }

    // Only semantic available.
    if keyword_results.is_empty() {
        let mut results = semantic_results;
        for r in &mut results {
            r.score = r.similarity;
        }
        results.truncate(limit);
        return Ok(results);
    }

    // --- RRF fusion ---
    let mut chunk_scores: HashMap<String, SearchResult> = HashMap::new();

    // Score keyword results.
    for (rank_idx, kr) in keyword_results.into_iter().enumerate() {
        let rank = rank_idx + 1; // 1-based
        let rrf_score = KEYWORD_WEIGHT / (RRF_K as f64 + rank as f64);
        chunk_scores.insert(
            kr.chunk_id.clone(),
            SearchResult {
                keyword_rank: Some(rank),
                score: rrf_score,
                ..kr
            },
        );
    }

    // Score semantic results, merging with keyword if present.
    for (rank_idx, sr) in semantic_results.into_iter().enumerate() {
        let rank = rank_idx + 1; // 1-based
        let rrf_score = SEMANTIC_WEIGHT / (RRF_K as f64 + rank as f64);

        if let Some(existing) = chunk_scores.get_mut(&sr.chunk_id) {
            existing.semantic_rank = Some(rank);
            existing.score += rrf_score;
        } else {
            chunk_scores.insert(
                sr.chunk_id.clone(),
                SearchResult {
                    semantic_rank: Some(rank),
                    score: rrf_score,
                    ..sr
                },
            );
        }
    }

    let mut ranked: Vec<SearchResult> = chunk_scores.into_values().collect();
    ranked.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    ranked.truncate(limit);
    Ok(ranked)
}
