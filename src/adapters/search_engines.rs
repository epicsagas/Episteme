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
    ("essence", 1.08),   // State, Command, etc. use "Essence" as the primary section
    ("structure", 1.04),
    // Law/principle-specific
    ("statement", 1.10), // DRY, KISS, and many laws use "Statement" as the primary section
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
            "shotgun",
            "surgery",
            "feature envy",
            "god object",
            "god class",
            "data clump",
            "primitive obsession",
            "divergent change",
            "parallel inheritance",
            "lazy class",
            "speculative",
            "temporary field",
            "message chain",
            "middle man",
            "inappropriate intimacy",
            "alternative classes",
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

/// Boost when a significant query word is an exact substring of the entity title.
///
/// For example, "state pattern" matches a title of "State", and "kiss principle"
/// matches a title of "Kiss".  This separates tied candidates that are scored
/// equally on cosine similarity alone.
///
/// Boosts are graduated:
///  - All non-stop-word query tokens appear in title → strongest boost (full title match)
///  - At least one non-stop-word query token appears → moderate boost
///  - No match → no boost
pub fn title_match_boost(title: &str, query_lower: &str) -> f64 {
    let title_lower = title.to_lowercase();
    // Generic words that are shared across many similar titles should not count
    // toward boosting (e.g. "extract" appears in Extract Method, Extract Class,
    // Extract Superclass…).  We use them only for counting, but require at least
    // one *specific* token to earn a boost.
    const STOP_WORDS: &[&str] = &["pattern", "principle", "smell", "refactoring", "law", "code"];

    let query_tokens: Vec<&str> = query_lower.split_whitespace().collect();
    let significant_tokens: Vec<&str> = query_tokens
        .iter()
        .filter(|t| !STOP_WORDS.contains(t))
        .copied()
        .collect();

    if significant_tokens.is_empty() {
        return 1.0;
    }

    let matching = significant_tokens
        .iter()
        .filter(|t| title_lower.contains(*t))
        .count();

    // Full title match (all significant tokens hit): strongest boost.
    // Partial match: moderate boost.
    if matching == significant_tokens.len() {
        // All significant query tokens appear in the title — this is a
        // near-exact name hit.  Use a stronger multiplier so that FTS5's
        // top-1 keyword result (e.g. "Shotgun Surgery" for query
        // "shotgun surgery smell") wins over semantic results that lack
        // these specific terms in their title.
        1.35
    } else if matching > 0 {
        1.10
    } else {
        1.0
    }
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
// Sparse entity title boost
// ---------------------------------------------------------------------------

/// Count how many chunks each entity has in the database.
fn count_chunks_per_entity(conn: &Connection) -> HashMap<String, usize> {
    let mut map = HashMap::new();
    let mut stmt = match conn.prepare("SELECT entity_id, COUNT(*) as cnt FROM chunks GROUP BY entity_id") {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("count_chunks_per_entity: prepare failed: {e}");
            return map;
        }
    };
    let rows = match stmt.query_map([], |row| {
        let entity_id: String = row.get(0)?;
        let cnt: i64 = row.get(1)?;
        let cnt: usize = cnt as usize;
        Ok((entity_id, cnt))
    }) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("count_chunks_per_entity: query_map failed: {e}");
            return map;
        }
    };
    for (eid, cnt) in rows.flatten() {
        map.insert(eid, cnt);
    }
    map
}

/// Return a boost multiplier for entities with very few indexed chunks.
///
/// Entities like "God Object" or "Shotgun Surgery" have brief overview
/// sections, resulting in only 1-2 chunks and depressed BM25 scores.  This
/// multiplier compensates so that their title matches are not unfairly
/// outranked by entities with many chunks.
pub fn sparse_entity_boost(counts: &HashMap<String, usize>, entity_id: &str) -> f64 {
    match counts.get(entity_id) {
        Some(&cnt) if cnt <= 2 => 1.3,
        Some(&cnt) if cnt <= 4 => 1.15,
        _ => 1.0,
    }
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
        // Extra boost when query terms appear directly in the title (e.g. "state pattern" → title "State").
        let title_boost = title_match_boost(&title, &query_lower);
        let score = similarity * sec_boost * t_boost * title_boost;

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
    keyword_search_with_chunk_counts(conn, query, limit, entity_type_filter, None)
}

/// Same as `keyword_search` but accepts a pre-computed chunk-count map.
/// When `chunk_counts` is `None`, it is computed from the database.
pub fn keyword_search_with_chunk_counts(
    conn: &Connection,
    query: &str,
    limit: usize,
    entity_type_filter: Option<&str>,
    chunk_counts: Option<HashMap<String, usize>>,
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

    // Re-rank: apply title match boost so that results whose title contains
    // the query's significant tokens rank higher than results that only matched
    // in body text.  FTS5 BM25 scores are negative (lower = better), so we
    // scale them into positive territory and then apply the boost multiplier.
    let query_lower = query.to_lowercase();
    let chunk_counts = chunk_counts.unwrap_or_else(|| count_chunks_per_entity(conn));
    for r in &mut results {
        let boost = title_match_boost(&r.title, &query_lower);
        let sparse_boost = sparse_entity_boost(&chunk_counts, &r.entity_id);
        // BM25 rank is negative; abs converts it to a positive relevance magnitude.
        r.score = r.score.abs() * boost * sparse_boost;
    }
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

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
const KEYWORD_WEIGHT: f64 = 0.45;
const SEMANTIC_WEIGHT: f64 = 0.55;

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

    // Compute chunk counts once for both keyword and RRF scoring.
    let chunk_counts = count_chunks_per_entity(conn);

    // --- keyword search (graceful degradation) ---
    let keyword_results: Vec<SearchResult> = keyword_search_with_chunk_counts(
        conn,
        query,
        expanded_limit,
        entity_type_filter,
        Some(chunk_counts.clone()),
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
    let query_lower_rrf = query.to_lowercase();
    let mut chunk_scores: HashMap<String, SearchResult> = HashMap::new();

    // Score keyword results.
    // Apply an additional title-match multiplier inside the RRF formula so
    // that exact-name matches (e.g. "Dry" for query "dry principle") rank
    // ahead of partial-body matches even after semantic fusion.
    for (rank_idx, kr) in keyword_results.into_iter().enumerate() {
        let rank = rank_idx + 1; // 1-based
        let t_boost = title_match_boost(&kr.title, &query_lower_rrf);
        let s_boost = section_boost(&kr.section);
        let sparse_boost = sparse_entity_boost(&chunk_counts, &kr.entity_id);
        let rrf_score = KEYWORD_WEIGHT / (RRF_K as f64 + rank as f64) * t_boost * s_boost * sparse_boost;
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

    // Sort chunks by score, then entity-level dedup: keep best chunk per entity.
    // This prevents entities with many chunks from unfairly crowding out entities
    // whose best chunk is more relevant (e.g. a single high-scoring chunk for a
    // specific smell beats many medium-scoring chunks for a generic one).
    let mut ranked: Vec<SearchResult> = chunk_scores.into_values().collect();
    ranked.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    let mut seen_entities = std::collections::HashSet::new();
    ranked.retain(|r| seen_entities.insert(r.entity_id.clone()));
    ranked.truncate(limit);
    Ok(ranked)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE chunks (
                id TEXT PRIMARY KEY,
                text TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                title TEXT NOT NULL,
                section TEXT NOT NULL,
                metadata TEXT NOT NULL DEFAULT '{}'
            );
            ",
        )
        .unwrap();
        conn
    }

    fn insert_chunk(conn: &Connection, id: &str, entity_id: &str, title: &str, text: &str) {
        conn.execute(
            "INSERT INTO chunks (id, text, entity_id, entity_type, title, section) VALUES (?1, ?2, ?3, 'pattern', ?4, 'overview')",
            params![id, text, entity_id, title],
        )
        .unwrap();
    }

    #[test]
    fn count_chunks_per_entity_basic() {
        let conn = setup_test_db();
        insert_chunk(&conn, "c1", "god_object", "God Object", "A god object...");
        insert_chunk(&conn, "c2", "god_object", "God Object", "Second chunk...");
        insert_chunk(&conn, "c3", "strategy", "Strategy", "Strategy pattern...");
        insert_chunk(&conn, "c4", "strategy", "Strategy", "Another...");
        insert_chunk(&conn, "c5", "strategy", "Strategy", "Third...");
        insert_chunk(&conn, "c6", "observer", "Observer", "Observer...");

        let counts = count_chunks_per_entity(&conn);
        assert_eq!(counts.get("god_object"), Some(&2));
        assert_eq!(counts.get("strategy"), Some(&3));
        assert_eq!(counts.get("observer"), Some(&1));
    }

    #[test]
    fn count_chunks_per_entity_empty_db() {
        let conn = setup_test_db();
        let counts = count_chunks_per_entity(&conn);
        assert!(counts.is_empty());
    }

    #[test]
    fn sparse_boost_tier_1_or_2_chunks() {
        let conn = setup_test_db();
        insert_chunk(&conn, "c1", "god_object", "God Object", "A god object...");
        insert_chunk(&conn, "c2", "god_object", "God Object", "Second chunk...");

        let counts = count_chunks_per_entity(&conn);
        let boost = sparse_entity_boost(&counts, "god_object");
        assert!((boost - 1.3).abs() < f64::EPSILON);
    }

    #[test]
    fn sparse_boost_tier_single_chunk() {
        let conn = setup_test_db();
        insert_chunk(&conn, "c1", "shotgun", "Shotgun Surgery", "Shotgun surgery...");

        let counts = count_chunks_per_entity(&conn);
        let boost = sparse_entity_boost(&counts, "shotgun");
        assert!((boost - 1.3).abs() < f64::EPSILON);
    }

    #[test]
    fn sparse_boost_tier_3_to_4_chunks() {
        let conn = setup_test_db();
        insert_chunk(&conn, "c1", "strategy", "Strategy", "one");
        insert_chunk(&conn, "c2", "strategy", "Strategy", "two");
        insert_chunk(&conn, "c3", "strategy", "Strategy", "three");

        let counts = count_chunks_per_entity(&conn);
        let boost = sparse_entity_boost(&counts, "strategy");
        assert!((boost - 1.15).abs() < f64::EPSILON);
    }

    #[test]
    fn sparse_boost_tier_4_chunks() {
        let conn = setup_test_db();
        insert_chunk(&conn, "c1", "observer", "Observer", "one");
        insert_chunk(&conn, "c2", "observer", "Observer", "two");
        insert_chunk(&conn, "c3", "observer", "Observer", "three");
        insert_chunk(&conn, "c4", "observer", "Observer", "four");

        let counts = count_chunks_per_entity(&conn);
        let boost = sparse_entity_boost(&counts, "observer");
        assert!((boost - 1.15).abs() < f64::EPSILON);
    }

    #[test]
    fn sparse_boost_tier_5_plus_chunks() {
        let conn = setup_test_db();
        for i in 0..6 {
            insert_chunk(
                &conn,
                &format!("c{i}"),
                "factory",
                "Factory",
                "chunk",
            );
        }

        let counts = count_chunks_per_entity(&conn);
        let boost = sparse_entity_boost(&counts, "factory");
        assert!((boost - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn sparse_boost_unknown_entity() {
        let conn = setup_test_db();
        insert_chunk(&conn, "c1", "strategy", "Strategy", "chunk");

        // Entity not in the DB at all should get 1.0
        let counts = count_chunks_per_entity(&conn);
        let boost = sparse_entity_boost(&counts, "nonexistent");
        assert!((boost - 1.0).abs() < f64::EPSILON);
    }
}
