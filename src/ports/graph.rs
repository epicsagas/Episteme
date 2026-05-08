use crate::domain::types::{
    Contradiction, CorrelationScore, Entity, GraphEdge, GraphStats, Neighborhood, UserEntity,
};
use std::collections::HashMap;

/// Trait for graph storage and traversal operations.
///
/// Implementations may be in-memory, database-backed, or remote.
pub trait GraphRepository: Send + Sync {
    /// Look up a single entity by id.
    fn get_entity(&self, id: &str) -> Option<&Entity>;

    /// Return a `{id: &Entity}` mapping for all requested IDs that exist.
    fn get_entities_batch(&self, ids: &[&str]) -> HashMap<String, &Entity>;

    /// Get all neighbor IDs of `entity_id`, optionally filtered by `relation_type`.
    fn get_neighbors(&self, entity_id: &str, relation_type: Option<&str>) -> Vec<String>;

    /// All outgoing edges from `entity_id`.
    fn get_all_edges(&self, entity_id: &str) -> Vec<GraphEdge>;

    /// Complete one-hop neighborhood: entity + outgoing + incoming edges.
    fn get_neighborhood(&self, id: &str) -> Option<Neighborhood>;

    /// BFS shortest path between `from_id` and `to_id` within `max_depth` hops.
    fn find_shortest_path(
        &self,
        from_id: &str,
        to_id: &str,
        max_depth: usize,
    ) -> Option<Vec<String>>;

    /// Find entities similar to `entity_id` using Jaccard similarity above `threshold`.
    fn find_similar_entities(&self, entity_id: &str, threshold: f64) -> Vec<(String, f64)>;

    /// Find entities that both enforce and violate the same principle.
    fn find_contradictions(&self) -> Vec<Contradiction>;

    /// Aggregate statistics about the loaded graph.
    fn stats(&self) -> GraphStats;

    /// Return all entity IDs in the graph.
    fn all_entity_ids(&self) -> Vec<String>;
}

/// Trait for mutable user-knowledge graph operations.
///
/// All mutation methods take `&self` — interior mutability (e.g. `Mutex`) is
/// handled by the adapter. This keeps the trait object-safe and compatible
/// with `Arc<dyn MutableGraphRepository>`.
pub trait MutableGraphRepository: Send + Sync {
    /// Add a new user entity. Returns an error if the ID already exists.
    fn add_entity(&self, entity: UserEntity) -> Result<(), String>;

    /// Update an existing user entity by ID.
    fn update_entity(&self, id: &str, entity: UserEntity) -> Result<(), String>;

    /// Remove a user entity and all its relations by ID.
    fn remove_entity(&self, id: &str) -> Result<(), String>;

    /// Add a directed relation between two entities.
    fn add_relation(&self, from: &str, relation: &str, to: &str) -> Result<(), String>;

    /// Remove a specific directed relation.
    fn remove_relation(&self, from: &str, relation: &str, to: &str) -> Result<(), String>;

    /// Look up a user entity by ID.
    fn get_user_entity(&self, id: &str) -> Option<UserEntity>;

    /// Get all user entity IDs.
    fn all_user_entity_ids(&self) -> Vec<String>;

    /// Get neighbors in the user graph (outgoing + incoming).
    fn get_user_neighbors(&self, entity_id: &str, relation_type: Option<&str>) -> Vec<String>;

    /// Get all outgoing edges from a user entity.
    fn get_user_all_edges(&self, entity_id: &str) -> Vec<GraphEdge>;

    /// Search user entities by keyword (FTS5-style).
    fn search_user_entities(&self, query: &str, limit: usize) -> Vec<UserEntity>;

    /// Load all user entities in a single query (avoids N+1).
    fn all_user_entities(&self) -> Vec<UserEntity>;

    /// Compute correlation scores between a target insight and all other insights.
    fn compute_correlations(&self, insight_id: &str) -> Vec<CorrelationScore>;

    /// Store an embedding vector for a user entity.
    fn store_embedding(&self, entity_id: &str, embedding: &[f32]) -> Result<(), String>;

    /// Retrieve the embedding vector for a user entity.
    fn get_embedding(&self, entity_id: &str) -> Option<Vec<f32>>;

    /// Return count of user entities.
    fn user_entity_count(&self) -> usize;
}
