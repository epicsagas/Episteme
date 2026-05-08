//! Thin MCP facade that delegates to focused service modules.
//!
//! Public API is identical to the original monolith; all behavior is forwarded
//! to `mcp_search`, `mcp_graph`, and `mcp_analysis`.

use std::sync::Mutex;
use std::time::Instant;

use crate::domain::graph::KnowledgeGraph;
use crate::ports::embeddings::EmbeddingProvider;

/// Main MCP handler that owns the knowledge graph and delegates to domain services.
///
/// Optionally holds a SQLite connection and embedding provider for hybrid RAG search.
/// When absent, falls back to pure keyword matching over graph entities.
pub struct EpistemeMCP {
    graph: KnowledgeGraph,
    db: Option<Mutex<rusqlite::Connection>>,
    embedding_provider: Option<Box<dyn EmbeddingProvider>>,
}

// -- constructors & accessors ------------------------------------------------

impl EpistemeMCP {
    pub fn new(graph: KnowledgeGraph) -> Self {
        Self {
            graph,
            db: None,
            embedding_provider: None,
        }
    }

    /// Create MCP handler with RAG support (SQLite + embedding provider).
    pub fn with_rag(
        graph: KnowledgeGraph,
        db: rusqlite::Connection,
        provider: Box<dyn EmbeddingProvider>,
    ) -> Self {
        Self {
            graph,
            db: Some(Mutex::new(db)),
            embedding_provider: Some(provider),
        }
    }

    /// Try to open the RAG database and attach it (best-effort, non-fatal).
    ///
    /// When configured for OpenAI and an API key is available,
    /// uses the OpenAI embedding provider. Otherwise falls back to
    /// local deterministic embeddings.
    pub fn try_attach_rag(&mut self) {
        let db_path = crate::adapters::paths::db_path();
        if !db_path.exists() {
            return;
        }
        // Ensure the FTS5 keyword-search index exists.  We open the database
        // read-write once just to build the index if it is missing, then
        // re-open read-only for normal operation.
        if let Ok(rw_conn) = rusqlite::Connection::open(&db_path) {
            let has_fts: bool = rw_conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='chunks_fts'",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0i64)
                > 0;

            if !has_fts {
                let _ = crate::adapters::search_engines::build_fts_index(&rw_conn);
            }
        }

        if let Ok(conn) = rusqlite::Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        ) {
            use crate::adapters::constants::EMBEDDING_DIMENSIONS;

            // Prefer OpenAI provider when configured and key is present.
            #[cfg(feature = "openai-embeddings")]
            {
                let cfg = crate::adapters::config::EpistemeConfig::load().unwrap_or_default();
                let provider_pref = cfg.embedding_provider.to_lowercase();
                let key = std::env::var("EPISTEME_OPENAI_API_KEY")
                    .ok()
                    .filter(|k| !k.is_empty())
                    .or_else(|| {
                        if cfg.openai_api_key.is_empty() {
                            None
                        } else {
                            Some(cfg.openai_api_key.clone())
                        }
                    });
                if provider_pref == "openai"
                    && let Some(key) = key
                {
                    let model = std::env::var("EPISTEME_OPENAI_EMBED_MODEL")
                        .ok()
                        .filter(|m| !m.is_empty())
                        .unwrap_or_else(|| cfg.openai_embed_model.clone());
                    let dim: usize = std::env::var("EPISTEME_OPENAI_EMBED_DIM")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(cfg.openai_embed_dim);
                    self.embedding_provider = Some(Box::new(
                        crate::adapters::openai_embeddings::OpenAIEmbeddingProvider::new(
                            key, model, dim,
                        ),
                    ));
                    self.db = Some(Mutex::new(conn));
                    return;
                }
            }

            // Fallback: local provider — instantiate now (cheap), model loads on first embed() call.
            self.embedding_provider = Some(Box::new(
                crate::adapters::local_embeddings::LocalEmbeddingProvider::new(
                    EMBEDDING_DIMENSIONS,
                ),
            ));
            self.db = Some(Mutex::new(conn));
            // Do NOT call warmup() here — let the first search request trigger model load
            // so MCP server startup is instant for the user.
        }
    }

    /// Access the underlying knowledge graph.
    pub fn graph(&self) -> &KnowledgeGraph {
        &self.graph
    }

    /// Check whether a RAG database is attached.
    pub fn has_db(&self) -> bool {
        self.db.is_some()
    }

    /// Check whether an embedding provider is configured.
    pub fn has_embedding_provider(&self) -> bool {
        self.embedding_provider.is_some()
    }
}

// -- tool implementations (delegating to service modules) ---------------------

impl EpistemeMCP {
    /// Search knowledge graph entities (delegates to `mcp_search`).
    pub fn search_knowledge(
        &self,
        query: &str,
        limit: Option<usize>,
        entity_type: Option<&str>,
    ) -> serde_json::Value {
        super::mcp_search::search_knowledge(
            &self.graph,
            self.db.as_ref(),
            self.embedding_provider.as_deref(),
            query,
            limit,
            entity_type,
        )
    }

    /// Get detailed information about a single entity (delegates to `mcp_graph`).
    pub fn get_entity(&self, entity_id: &str, detail_level: Option<&str>) -> serde_json::Value {
        super::mcp_graph::get_entity(&self.graph, entity_id, detail_level)
    }

    /// Get entities related to a given entity (delegates to `mcp_graph`).
    pub fn get_neighbors(&self, entity_id: &str, relation_type: Option<&str>) -> serde_json::Value {
        super::mcp_graph::get_neighbors(&self.graph, entity_id, relation_type)
    }

    /// Find the shortest path between two entities (delegates to `mcp_graph`).
    pub fn find_path(
        &self,
        from_id: &str,
        to_id: &str,
        max_depth: Option<usize>,
    ) -> serde_json::Value {
        super::mcp_graph::find_path(&self.graph, from_id, to_id, max_depth)
    }

    /// Detect code smells (delegates to `mcp_analysis`).
    pub fn analyze_code(&self, code: &str, language: Option<&str>) -> serde_json::Value {
        super::mcp_analysis::analyze_code(code, language)
    }

    /// Detect smells and suggest ranked refactorings (delegates to `mcp_analysis`).
    pub fn suggest_refactorings(
        &self,
        code: &str,
        language: Option<&str>,
        top_k: Option<usize>,
    ) -> serde_json::Value {
        super::mcp_analysis::suggest_refactorings(&self.graph, code, language, top_k)
    }
}

// -- resource implementations ------------------------------------------------

impl EpistemeMCP {
    pub fn handle_resource_read(&self, uri: &str) -> serde_json::Value {
        match uri {
            "episteme://stats" => {
                let stats = self.graph.stats();
                serde_json::to_value(stats)
                    .unwrap_or(serde_json::json!({"error": "serialization failed"}))
            }
            "episteme://categories" => {
                let entity_types: Vec<&str> = ["pattern", "refactoring", "law", "smell"].to_vec();
                let categories: Vec<&str> = [
                    "teams",
                    "planning",
                    "architecture",
                    "quality",
                    "scalability",
                    "design",
                    "decisions",
                ]
                .to_vec();
                serde_json::json!({
                    "entity_types": entity_types,
                    "categories": categories,
                })
            }
            "episteme://contradictions" => {
                let contradictions = self.graph.find_contradictions();
                serde_json::json!(contradictions)
            }
            _ => serde_json::json!({
                "error": format!("Unknown resource '{}'.", uri)
            }),
        }
    }
}

// -- tool dispatch -----------------------------------------------------------

impl EpistemeMCP {
    /// Top-level tool dispatch: route a tool name + arguments to the right method.
    pub fn handle_tool_call(&self, name: &str, args: &serde_json::Value) -> serde_json::Value {
        let telemetry_tool = Self::telemetry_tool_for(name);
        if let Some(tool) = telemetry_tool {
            crate::adapters::telemetry::track_tool_called(tool);
        }
        let started_at = Instant::now();

        let result = match name {
            "search_knowledge" => self.dispatch_search_knowledge(args),
            "get_entity" => self.dispatch_get_entity(args),
            "get_neighbors" => self.dispatch_get_neighbors(args),
            "find_path" => self.dispatch_find_path(args),
            "analyze_code" => self.dispatch_analyze_code(args),
            "suggest_refactorings" => self.dispatch_suggest_refactorings(args),
            _ => serde_json::json!({
                "error": format!("Unknown tool '{}'.", name)
            }),
        };

        if let Some(tool) = telemetry_tool {
            Self::record_telemetry(tool, &result, started_at);
        }

        result
    }

    // -- individual tool dispatch helpers ------------------------------------

    fn dispatch_search_knowledge(&self, args: &serde_json::Value) -> serde_json::Value {
        let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
        let limit = args
            .get("limit")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);
        let entity_type = args.get("entity_type").and_then(|v| v.as_str());
        self.search_knowledge(query, limit, entity_type)
    }

    fn dispatch_get_entity(&self, args: &serde_json::Value) -> serde_json::Value {
        let entity_id = args.get("entity_id").and_then(|v| v.as_str()).unwrap_or("");
        let detail_level = args.get("detail_level").and_then(|v| v.as_str());
        self.get_entity(entity_id, detail_level)
    }

    fn dispatch_get_neighbors(&self, args: &serde_json::Value) -> serde_json::Value {
        let entity_id = args.get("entity_id").and_then(|v| v.as_str()).unwrap_or("");
        let relation_type = args.get("relation_type").and_then(|v| v.as_str());
        self.get_neighbors(entity_id, relation_type)
    }

    fn dispatch_find_path(&self, args: &serde_json::Value) -> serde_json::Value {
        let from_id = args.get("from_id").and_then(|v| v.as_str()).unwrap_or("");
        let to_id = args.get("to_id").and_then(|v| v.as_str()).unwrap_or("");
        let max_depth = args
            .get("max_depth")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);
        self.find_path(from_id, to_id, max_depth)
    }

    fn dispatch_analyze_code(&self, args: &serde_json::Value) -> serde_json::Value {
        let code = args.get("code").and_then(|v| v.as_str()).unwrap_or("");
        let language = args.get("language").and_then(|v| v.as_str());
        self.analyze_code(code, language)
    }

    fn dispatch_suggest_refactorings(&self, args: &serde_json::Value) -> serde_json::Value {
        let code = args.get("code").and_then(|v| v.as_str()).unwrap_or("");
        let language = args.get("language").and_then(|v| v.as_str());
        let top_k = args
            .get("top_k")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);
        self.suggest_refactorings(code, language, top_k)
    }

    // -- telemetry helpers ---------------------------------------------------

    fn telemetry_tool_for(name: &str) -> Option<crate::adapters::telemetry::Tool> {
        match name {
            "search_knowledge" => Some(crate::adapters::telemetry::Tool::SearchKnowledge),
            "get_entity" => Some(crate::adapters::telemetry::Tool::GetEntity),
            "get_neighbors" => Some(crate::adapters::telemetry::Tool::GetNeighbors),
            "find_path" => Some(crate::adapters::telemetry::Tool::FindPath),
            "analyze_code" => Some(crate::adapters::telemetry::Tool::AnalyzeCode),
            "suggest_refactorings" => Some(crate::adapters::telemetry::Tool::SuggestRefactorings),
            _ => None,
        }
    }

    fn record_telemetry(
        tool: crate::adapters::telemetry::Tool,
        result: &serde_json::Value,
        started_at: Instant,
    ) {
        if result.get("error").is_some() {
            crate::adapters::telemetry::track_tool_failed(
                tool,
                crate::adapters::telemetry::FailureClass::Unknown,
            );
        } else {
            let count = result
                .get("count")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize)
                .unwrap_or_else(|| {
                    if result.get("results").and_then(|v| v.as_array()).is_some() {
                        result["results"].as_array().map(|a| a.len()).unwrap_or(0)
                    } else if result.get("smells").and_then(|v| v.as_array()).is_some() {
                        result["smells"].as_array().map(|a| a.len()).unwrap_or(0)
                    } else if result.get("analyses").and_then(|v| v.as_array()).is_some() {
                        result["analyses"].as_array().map(|a| a.len()).unwrap_or(0)
                    } else {
                        1
                    }
                });
            crate::adapters::telemetry::track_tool_completed(
                tool,
                started_at.elapsed().as_millis(),
                count.into(),
            );
        }
    }
}
