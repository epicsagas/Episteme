use crate::domain::types::{Contradiction, Entity, GraphEdge, GraphStats, Neighborhood};
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
