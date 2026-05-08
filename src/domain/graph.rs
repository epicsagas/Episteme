use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

use tracing;

use crate::domain::types::*;
use crate::ports::graph::GraphRepository;

// ---------------------------------------------------------------------------
// Error type (domain-only — no thiserror dependency)
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum GraphError {
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl std::fmt::Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphError::Io(e) => write!(f, "IO error reading relations: {e}"),
            GraphError::Json(e) => write!(f, "JSON parse error: {e}"),
        }
    }
}

impl std::error::Error for GraphError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            GraphError::Io(e) => Some(e),
            GraphError::Json(e) => Some(e),
        }
    }
}

impl From<std::io::Error> for GraphError {
    fn from(e: std::io::Error) -> Self {
        GraphError::Io(e)
    }
}

impl From<serde_json::Error> for GraphError {
    fn from(e: serde_json::Error) -> Self {
        GraphError::Json(e)
    }
}

pub type Result<T> = std::result::Result<T, GraphError>;

// Prefixes that identify valid entity keys in relations.json.
const ENTITY_PREFIXES: &[&str] = &["DP-", "RF-", "LAW-", "SMELL-"];

// ---------------------------------------------------------------------------
// Knowledge graph
// ---------------------------------------------------------------------------

/// Knowledge graph traversal and analysis engine.
///
/// Ported from `episteme.graph.api.KnowledgeGraph` (Python).
/// The graph is constructed via `from_entities()` and then queried via `&self` methods.
pub struct KnowledgeGraph {
    /// Entity id -> raw entity object.
    pub(crate) entities: HashMap<String, Entity>,

    /// Reverse index: target id -> relation type -> source ids.
    reverse_relations: HashMap<String, HashMap<String, Vec<String>>>,
}

impl Clone for KnowledgeGraph {
    fn clone(&self) -> Self {
        Self {
            entities: self.entities.clone(),
            reverse_relations: self.reverse_relations.clone(),
        }
    }
}

impl KnowledgeGraph {
    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    /// Build a knowledge graph from an existing HashMap of entities (pure, no I/O).
    ///
    /// Derives `solved_by` from forward `solves` edges so that `solves` is the
    /// single source of truth. Any stale `solved_by` in the data is overwritten.
    pub fn from_entities(mut entities: HashMap<String, Entity>) -> Self {
        Self::derive_inverse_relations(&mut entities, "solves", "solved_by");

        let reverse_relations = Self::build_reverse_index(&entities);

        tracing::info!(
            entities = entities.len(),
            "constructed knowledge graph from entities"
        );

        Self {
            entities,
            reverse_relations,
        }
    }

    /// Scan all entities for `forward_key` edges and inject the corresponding
    /// `inverse_key` into each target entity, overwriting any existing value.
    fn derive_inverse_relations(
        entities: &mut HashMap<String, Entity>,
        forward_key: &str,
        inverse_key: &str,
    ) {
        let mut inverse_map: HashMap<String, Vec<String>> = HashMap::new();

        for (entity_id, entity) in entities.iter() {
            if let Some(targets) = entity.relations.get(forward_key) {
                for target_id in targets {
                    inverse_map
                        .entry(target_id.clone())
                        .or_default()
                        .push(entity_id.clone());
                }
            }
        }

        for (target_id, mut sources) in inverse_map {
            sources.sort();
            sources.dedup();
            if let Some(entity) = entities.get_mut(&target_id) {
                entity
                    .relations
                    .insert(inverse_key.to_owned(), sources);
            }
        }
    }

    /// Load the knowledge graph from `data_dir`, which must contain
    /// a `relations.json` file.
    ///
    /// The JSON file may contain non-entity keys (e.g. `"version"`).
    /// Only entries whose key starts with a known entity prefix
    /// (`DP-`, `RF-`, `LAW-`, `SMELL-`) are kept; everything else
    /// is silently ignored.
    pub fn load(data_dir: &Path) -> Result<Self> {
        let relations_path = data_dir.join("relations.json");
        let raw = std::fs::read_to_string(&relations_path)?;

        let json_map: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&raw)?;

        let mut entities = HashMap::new();

        for (key, value) in json_map {
            if !ENTITY_PREFIXES.iter().any(|prefix| key.starts_with(prefix)) {
                continue;
            }
            match serde_json::from_value::<Entity>(value) {
                Ok(mut entity) => {
                    entity.id = key.clone();
                    entities.insert(key, entity);
                }
                Err(_e) => {
                    // Skip malformed entities silently
                }
            }
        }

        tracing::info!(
            entities = entities.len(),
            "loaded knowledge graph from {}",
            relations_path.display()
        );

        Ok(Self::from_entities(entities))
    }

    fn build_reverse_index(
        entities: &HashMap<String, Entity>,
    ) -> HashMap<String, HashMap<String, Vec<String>>> {
        let mut reverse: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();

        for (entity_id, entity) in entities {
            for (rel_type, targets) in &entity.relations {
                for target_id in targets {
                    reverse
                        .entry(target_id.clone())
                        .or_default()
                        .entry(rel_type.clone())
                        .or_default()
                        .push(entity_id.clone());
                }
            }
        }

        reverse
    }

    // -----------------------------------------------------------------------
    // Basic queries
    // -----------------------------------------------------------------------

    /// Look up a single entity by id.
    pub fn get_entity(&self, id: &str) -> Option<&Entity> {
        self.entities.get(id)
    }

    /// Return a `{id: &Entity}` mapping for all requested IDs that exist.
    pub fn get_entities_batch(&self, ids: &[&str]) -> HashMap<String, &Entity> {
        ids.iter()
            .filter_map(|id| self.entities.get(*id).map(|e| ((*id).to_owned(), e)))
            .collect()
    }

    /// Get all neighbor IDs of `entity_id`, optionally filtered by `relation_type`.
    pub fn get_neighbors(&self, entity_id: &str, relation_type: Option<&str>) -> Vec<String> {
        let Some(entity) = self.entities.get(entity_id) else {
            return Vec::new();
        };

        if let Some(rt) = relation_type {
            entity.relations.get(rt).cloned().unwrap_or_default()
        } else {
            let mut seen = HashSet::new();
            let mut neighbors = Vec::new();
            for targets in entity.relations.values() {
                for t in targets {
                    if seen.insert(t.clone()) {
                        neighbors.push(t.clone());
                    }
                }
            }
            neighbors
        }
    }

    /// All outgoing edges from `entity_id`.
    pub fn get_all_edges(&self, entity_id: &str) -> Vec<GraphEdge> {
        let Some(entity) = self.entities.get(entity_id) else {
            return Vec::new();
        };

        let mut edges = Vec::new();
        for (rel_type, targets) in &entity.relations {
            for target_id in targets {
                edges.push(GraphEdge {
                    from_id: entity_id.to_owned(),
                    to_id: target_id.clone(),
                    relation_type: rel_type.clone(),
                });
            }
        }
        edges
    }

    /// Complete one-hop neighborhood: entity + outgoing + incoming edges.
    pub fn get_neighborhood(&self, id: &str) -> Option<Neighborhood> {
        let entity = self.entities.get(id)?;

        let outgoing = entity.relations.clone();
        let incoming = self.reverse_relations.get(id).cloned().unwrap_or_default();

        Some(Neighborhood {
            entity: entity.clone(),
            outgoing,
            incoming,
        })
    }

    // -----------------------------------------------------------------------
    // Multi-hop queries
    // -----------------------------------------------------------------------

    /// Traverse a chain of relation types starting from `start`.
    ///
    /// Returns all paths found. Each path is a list of entity ids.
    /// For example `traverse_chain("SMELL-01", &["solved_by", "enforces"])`
    /// may return `[["SMELL-01", "RF-001", "LAW-042-S"], ...]`.
    pub fn traverse_chain(&self, start: &str, path: &[&str]) -> Vec<Vec<String>> {
        if path.is_empty() {
            return vec![vec![start.to_owned()]];
        }

        let mut current_paths: Vec<Vec<String>> = vec![vec![start.to_owned()]];

        for rel_type in path {
            let mut next_paths = Vec::new();
            for current_path in &current_paths {
                let current_id = current_path.last().unwrap();
                let neighbors = self.get_neighbors(current_id, Some(rel_type));
                for neighbor_id in neighbors {
                    let mut extended = current_path.clone();
                    extended.push(neighbor_id);
                    next_paths.push(extended);
                }
            }
            current_paths = next_paths;
        }

        current_paths
    }

    /// BFS shortest path between `from_id` and `to_id`.
    ///
    /// Returns `None` if no path exists within `max_depth` hops.
    pub fn find_shortest_path(
        &self,
        from_id: &str,
        to_id: &str,
        max_depth: usize,
    ) -> Option<Vec<String>> {
        if from_id == to_id {
            return Some(vec![from_id.to_owned()]);
        }

        if !self.entities.contains_key(from_id) || !self.entities.contains_key(to_id) {
            return None;
        }

        let mut queue: VecDeque<(String, Vec<String>)> = VecDeque::new();
        queue.push_back((from_id.to_owned(), vec![from_id.to_owned()]));

        let mut visited: HashSet<String> = HashSet::new();
        visited.insert(from_id.to_owned());

        while let Some((current_id, path)) = queue.pop_front() {
            if path.len() > max_depth {
                continue;
            }

            let neighbors = self.get_neighbors(&current_id, None);

            for neighbor_id in neighbors {
                if neighbor_id == to_id {
                    let mut result = path.clone();
                    result.push(neighbor_id);
                    return Some(result);
                }

                if visited.insert(neighbor_id.clone()) {
                    let mut new_path = path.clone();
                    new_path.push(neighbor_id.clone());
                    queue.push_back((neighbor_id, new_path));
                }
            }
        }

        None
    }

    /// Extract the subgraph within `radius` hops of `center_id`.
    ///
    /// Returns `(nodes, edges)`.
    pub fn extract_subgraph(
        &self,
        center_id: &str,
        radius: usize,
    ) -> (HashSet<String>, Vec<GraphEdge>) {
        if !self.entities.contains_key(center_id) {
            return (HashSet::new(), Vec::new());
        }

        let mut nodes = HashSet::new();
        nodes.insert(center_id.to_owned());

        let mut edges = Vec::new();
        let mut current_layer: HashSet<String> = HashSet::new();
        current_layer.insert(center_id.to_owned());

        for _ in 0..radius {
            let mut next_layer = HashSet::new();

            for node_id in &current_layer {
                let node_edges = self.get_all_edges(node_id);

                for edge in node_edges {
                    if self.entities.contains_key(&edge.to_id) {
                        nodes.insert(edge.to_id.clone());
                        next_layer.insert(edge.to_id.clone());
                        edges.push(edge);
                    }
                }
            }

            current_layer = next_layer;
        }

        (nodes, edges)
    }

    // -----------------------------------------------------------------------
    // Advanced analysis
    // -----------------------------------------------------------------------

    /// Find entities that both enforce *and* violate the same principle.
    pub fn find_contradictions(&self) -> Vec<Contradiction> {
        let mut contradictions = Vec::new();

        for (entity_id, entity) in &self.entities {
            let enforces: HashSet<&str> = entity
                .relations
                .get("enforces")
                .map(|v| v.iter().map(|s| s.as_str()).collect())
                .unwrap_or_default();

            let violates: HashSet<&str> = entity
                .relations
                .get("violates")
                .map(|v| v.iter().map(|s| s.as_str()).collect())
                .unwrap_or_default();

            let conflicts: Vec<String> = enforces
                .intersection(&violates)
                .map(|s| s.to_string())
                .collect();

            if !conflicts.is_empty() {
                let title = if !entity.title.is_empty() {
                    entity.title.clone()
                } else if !entity.name.is_empty() {
                    entity.name.clone()
                } else {
                    "Unknown".to_owned()
                };

                contradictions.push(Contradiction {
                    entity_id: entity_id.clone(),
                    title,
                    conflicts,
                });
            }
        }

        contradictions
    }

    /// Infer transitive enforcement relationships.
    ///
    /// Logic: If an RF solves a SMELL, and that SMELL violates a LAW,
    /// then the RF enforces the LAW (if not already an explicit relation).
    ///
    /// Returns `Vec<(rf_id, smell_id, law_id)>`.
    pub fn infer_transitive_enforcements(&self) -> Vec<(String, String, String)> {
        let mut inferred = Vec::new();

        for (rf_id, rf_entity) in &self.entities {
            if !rf_id.starts_with("RF-") {
                continue;
            }

            let solved_smells = match rf_entity.relations.get("solves") {
                Some(v) => v,
                None => continue,
            };

            let explicit_enforces: HashSet<&str> = rf_entity
                .relations
                .get("enforces")
                .map(|v| v.iter().map(|s| s.as_str()).collect())
                .unwrap_or_default();

            for smell_id in solved_smells {
                let Some(smell_entity) = self.entities.get(smell_id) else {
                    continue;
                };

                let violated_laws = match smell_entity.relations.get("violates") {
                    Some(v) => v,
                    None => continue,
                };

                for law_id in violated_laws {
                    if !explicit_enforces.contains(law_id.as_str()) {
                        inferred.push((rf_id.clone(), smell_id.clone(), law_id.clone()));
                    }
                }
            }
        }

        inferred
    }

    /// Find entities similar to `entity_id` using Jaccard similarity on
    /// the set of `"rel_type:target"` edge labels.
    ///
    /// Only pairs with similarity >= `threshold` are returned, sorted
    /// descending by score.
    pub fn find_similar_entities(&self, entity_id: &str, threshold: f64) -> Vec<(String, f64)> {
        let Some(entity) = self.entities.get(entity_id) else {
            return Vec::new();
        };

        let entity_edges: HashSet<String> = Self::edge_set(entity);

        let mut similar = Vec::new();

        for (other_id, other_entity) in &self.entities {
            if other_id == entity_id {
                continue;
            }

            let other_edges = Self::edge_set(other_entity);

            if entity_edges.is_empty() && other_edges.is_empty() {
                continue;
            }

            let intersection = entity_edges.intersection(&other_edges).count();
            let union = entity_edges.union(&other_edges).count();

            if union > 0 {
                let similarity = intersection as f64 / union as f64;

                if similarity >= threshold {
                    similar.push((other_id.clone(), similarity));
                }
            }
        }

        similar.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        similar
    }

    /// Build the set of `"rel_type:target"` strings for Jaccard comparison.
    fn edge_set(entity: &Entity) -> HashSet<String> {
        let mut set = HashSet::new();
        for (rel_type, targets) in &entity.relations {
            for target in targets {
                set.insert(format!("{rel_type}:{target}"));
            }
        }
        set
    }

    // -----------------------------------------------------------------------
    // Statistics
    // -----------------------------------------------------------------------

    /// Aggregate statistics about the loaded graph.
    pub fn stats(&self) -> GraphStats {
        let total_entities = self.entities.len();

        let mut by_type: HashMap<String, usize> = HashMap::new();
        let mut total_edges = 0usize;

        for entity in self.entities.values() {
            let t = if entity.r#type.is_empty() {
                "unknown"
            } else {
                &entity.r#type
            };
            *by_type.entry(t.to_owned()).or_insert(0) += 1;

            for targets in entity.relations.values() {
                total_edges += targets.len();
            }
        }

        let entities_with_relations = self
            .entities
            .values()
            .filter(|e| e.relations.values().any(|v| !v.is_empty()))
            .count();

        let avg_edges_per_entity = if total_entities > 0 {
            total_edges as f64 / total_entities as f64
        } else {
            0.0
        };

        GraphStats {
            total_entities,
            total_edges,
            by_type,
            entities_with_relations,
            avg_edges_per_entity,
        }
    }

    /// Return all entity IDs in the graph.
    pub fn all_entity_ids(&self) -> Vec<String> {
        self.entities.keys().cloned().collect()
    }
}

// ---------------------------------------------------------------------------
// GraphRepository trait implementation
// ---------------------------------------------------------------------------

impl GraphRepository for KnowledgeGraph {
    fn get_entity(&self, id: &str) -> Option<&Entity> {
        self.entities.get(id)
    }

    fn get_entities_batch(&self, ids: &[&str]) -> HashMap<String, &Entity> {
        ids.iter()
            .filter_map(|id| self.entities.get(*id).map(|e| ((*id).to_owned(), e)))
            .collect()
    }

    fn get_neighbors(&self, entity_id: &str, relation_type: Option<&str>) -> Vec<String> {
        self.get_neighbors(entity_id, relation_type)
    }

    fn get_all_edges(&self, entity_id: &str) -> Vec<GraphEdge> {
        self.get_all_edges(entity_id)
    }

    fn get_neighborhood(&self, id: &str) -> Option<Neighborhood> {
        self.get_neighborhood(id)
    }

    fn find_shortest_path(
        &self,
        from_id: &str,
        to_id: &str,
        max_depth: usize,
    ) -> Option<Vec<String>> {
        self.find_shortest_path(from_id, to_id, max_depth)
    }

    fn find_similar_entities(&self, entity_id: &str, threshold: f64) -> Vec<(String, f64)> {
        self.find_similar_entities(entity_id, threshold)
    }

    fn find_contradictions(&self) -> Vec<Contradiction> {
        self.find_contradictions()
    }

    fn stats(&self) -> GraphStats {
        self.stats()
    }

    fn all_entity_ids(&self) -> Vec<String> {
        self.all_entity_ids()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Helper: minimal Entity with all fields populated.
    pub(crate) fn blank_entity(id: &str) -> Entity {
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

    /// Build a `KnowledgeGraph` from a vec of entities using `from_entities()`.
    pub(crate) fn build_graph_from_entities(entities: Vec<Entity>) -> KnowledgeGraph {
        let map: HashMap<String, Entity> =
            entities.into_iter().map(|e| (e.id.clone(), e)).collect();
        KnowledgeGraph::from_entities(map)
    }

    #[test]
    fn from_entities_constructs_graph() {
        let mut smell = blank_entity("SMELL-01");
        smell.title = "Long Method".to_owned();
        smell.r#type = "smell".to_owned();
        smell
            .relations
            .insert("solved_by".to_owned(), vec!["RF-001".to_owned()]);

        let mut rf = blank_entity("RF-001");
        rf.title = "Extract Method".to_owned();
        rf.r#type = "refactoring".to_owned();

        let kg = build_graph_from_entities(vec![smell, rf]);
        assert_eq!(kg.stats().total_entities, 2);

        // Check neighbors work
        let neighbors = kg.get_neighbors("SMELL-01", Some("solved_by"));
        assert_eq!(neighbors, vec!["RF-001".to_owned()]);
    }

    #[test]
    fn all_entity_ids_returns_all_keys() {
        let mut e1 = blank_entity("DP-001");
        e1.title = "Singleton".to_owned();
        let mut e2 = blank_entity("DP-002");
        e2.title = "Factory".to_owned();

        let kg = build_graph_from_entities(vec![e1, e2]);
        let mut ids = kg.all_entity_ids();
        ids.sort();
        assert_eq!(ids, vec!["DP-001".to_owned(), "DP-002".to_owned()]);
    }

    #[test]
    fn graph_repository_trait_is_implemented() {
        fn assert_graph_repo<T: GraphRepository>(_: &T) {}
        let kg = KnowledgeGraph::from_entities(HashMap::new());
        assert_graph_repo(&kg);
    }

    #[test]
    fn get_entity_returns_known_id() {
        let mut e = blank_entity("SMELL-01");
        e.title = "Long Method".to_owned();
        let kg = build_graph_from_entities(vec![e]);
        let found = kg.get_entity("SMELL-01").expect("entity should exist");
        assert_eq!(found.id, "SMELL-01");
    }

    #[test]
    fn get_entity_unknown_returns_none() {
        let kg = build_graph_from_entities(vec![]);
        assert!(kg.get_entity("NONEXISTENT-999").is_none());
    }

    #[test]
    fn get_entities_batch_basic() {
        let e1 = blank_entity("DP-001");
        let e2 = blank_entity("DP-002");
        let e3 = blank_entity("DP-003");
        let kg = build_graph_from_entities(vec![e1, e2, e3]);
        let batch = kg.get_entities_batch(&["DP-001", "DP-002", "DP-003"]);
        assert_eq!(batch.len(), 3);
    }

    #[test]
    fn get_neighbors_deduplicates() {
        let mut e = blank_entity("DP-001");
        e.relations.insert(
            "related_to".to_owned(),
            vec!["DP-002".to_owned(), "DP-002".to_owned()],
        );
        let kg = build_graph_from_entities(vec![e]);
        let neighbors = kg.get_neighbors("DP-001", None);
        // Should not contain duplicates
        let mut seen = HashSet::new();
        for n in &neighbors {
            assert!(seen.insert(n.clone()), "duplicate neighbor {n}");
        }
    }

    #[test]
    fn get_all_edges_roundtrip() {
        let mut e = blank_entity("SMELL-01");
        e.relations
            .insert("solved_by".to_owned(), vec!["RF-001".to_owned()]);
        let kg = build_graph_from_entities(vec![e]);
        let edges = kg.get_all_edges("SMELL-01");
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].from_id, "SMELL-01");
        assert_eq!(edges[0].to_id, "RF-001");
        assert_eq!(edges[0].relation_type, "solved_by");
    }

    #[test]
    fn get_neighborhood_returns_incoming() {
        let mut smell = blank_entity("SMELL-01");
        smell
            .relations
            .insert("solved_by".to_owned(), vec!["RF-001".to_owned()]);
        let rf = blank_entity("RF-001");
        let kg = build_graph_from_entities(vec![smell, rf]);

        let nb = kg.get_neighborhood("RF-001").expect("should exist");
        assert_eq!(nb.entity.id, "RF-001");
        // Incoming should contain the reverse relation from SMELL-01
        assert!(!nb.incoming.is_empty(), "RF-001 should have incoming edges");
    }

    #[test]
    fn traverse_chain_single_step() {
        let mut smell = blank_entity("SMELL-01");
        smell
            .relations
            .insert("solved_by".to_owned(), vec!["RF-001".to_owned()]);
        let rf = blank_entity("RF-001");
        let kg = build_graph_from_entities(vec![smell, rf]);

        let paths = kg.traverse_chain("SMELL-01", &["solved_by"]);
        assert!(!paths.is_empty(), "should find at least one path");
        assert_eq!(paths[0][0], "SMELL-01");
        assert!(paths[0].len() >= 2);
    }

    #[test]
    fn find_shortest_path_identity() {
        let e = blank_entity("DP-001");
        let kg = build_graph_from_entities(vec![e]);
        let path = kg
            .find_shortest_path("DP-001", "DP-001", 5)
            .expect("same node");
        assert_eq!(path, vec!["DP-001".to_owned()]);
    }

    #[test]
    fn find_shortest_path_between_two_nodes() {
        let mut e1 = blank_entity("DP-001");
        e1.relations
            .insert("related_to".to_owned(), vec!["DP-002".to_owned()]);
        let e2 = blank_entity("DP-002");
        let kg = build_graph_from_entities(vec![e1, e2]);

        let path = kg
            .find_shortest_path("DP-001", "DP-002", 6)
            .expect("path should exist");
        assert_eq!(path.first().unwrap(), "DP-001");
        assert_eq!(path.last().unwrap(), "DP-002");
    }

    #[test]
    fn extract_subgraph_radius_1() {
        let mut e1 = blank_entity("DP-001");
        e1.relations
            .insert("related_to".to_owned(), vec!["DP-002".to_owned()]);
        let e2 = blank_entity("DP-002");
        let kg = build_graph_from_entities(vec![e1, e2]);

        let (nodes, edges) = kg.extract_subgraph("DP-001", 1);
        assert!(nodes.contains("DP-001"));
        assert!(nodes.contains("DP-002"));
        assert_eq!(edges.len(), 1);
    }

    #[test]
    fn stats_counts_are_consistent() {
        let mut e1 = blank_entity("DP-001");
        e1.r#type = "pattern".to_owned();
        e1.relations
            .insert("related_to".to_owned(), vec!["DP-002".to_owned()]);
        let mut e2 = blank_entity("RF-001");
        e2.r#type = "refactoring".to_owned();
        let kg = build_graph_from_entities(vec![e1, e2]);

        let s = kg.stats();
        assert_eq!(s.total_entities, 2);
        let type_sum: usize = s.by_type.values().sum();
        assert_eq!(type_sum, s.total_entities);
        assert!(s.avg_edges_per_entity >= 0.0);
    }

    #[test]
    fn find_similar_entities_with_low_threshold() {
        let mut e1 = blank_entity("DP-001");
        e1.relations
            .insert("related_to".to_owned(), vec!["DP-003".to_owned()]);
        let mut e2 = blank_entity("DP-002");
        e2.relations
            .insert("related_to".to_owned(), vec!["DP-003".to_owned()]);
        let e3 = blank_entity("DP-003");
        let kg = build_graph_from_entities(vec![e1, e2, e3]);

        let similar = kg.find_similar_entities("DP-001", 0.0);
        // DP-002 has the same edge pattern, should be similar
        assert!(!similar.is_empty());
    }

    #[test]
    fn infer_transitive_enforcements_runs() {
        let mut smell = blank_entity("SMELL-01");
        smell
            .relations
            .insert("violates".to_owned(), vec!["LAW-001".to_owned()]);
        let mut rf = blank_entity("RF-001");
        rf.relations
            .insert("solves".to_owned(), vec!["SMELL-01".to_owned()]);
        let law = blank_entity("LAW-001");
        let kg = build_graph_from_entities(vec![smell, rf, law]);

        let inferred = kg.infer_transitive_enforcements();
        assert_eq!(inferred.len(), 1);
        let (rf_id, smell_id, law_id) = &inferred[0];
        assert_eq!(rf_id, "RF-001");
        assert_eq!(smell_id, "SMELL-01");
        assert_eq!(law_id, "LAW-001");
    }

    #[test]
    fn find_contradictions_runs() {
        let mut e = blank_entity("DP-001");
        e.relations
            .insert("enforces".to_owned(), vec!["LAW-001".to_owned()]);
        e.relations
            .insert("violates".to_owned(), vec!["LAW-001".to_owned()]);
        let kg = build_graph_from_entities(vec![e]);

        let contradictions = kg.find_contradictions();
        assert_eq!(contradictions.len(), 1);
        assert_eq!(contradictions[0].conflicts, vec!["LAW-001".to_owned()]);
    }

    #[test]
    fn derive_solved_by_from_solves() {
        // RF-001 solves SMELL-01 and SMELL-02
        let mut rf1 = blank_entity("RF-001");
        rf1.relations
            .insert("solves".to_owned(), vec!["SMELL-01".to_owned()]);
        let mut rf2 = blank_entity("RF-002");
        rf2.relations
            .insert("solves".to_owned(), vec!["SMELL-01".to_owned(), "SMELL-02".to_owned()]);

        let smell1 = blank_entity("SMELL-01");
        let smell2 = blank_entity("SMELL-02");

        let kg = build_graph_from_entities(vec![rf1, rf2, smell1, smell2]);

        let solved_by_1 = kg.get_neighbors("SMELL-01", Some("solved_by"));
        let solved_by_2 = kg.get_neighbors("SMELL-02", Some("solved_by"));

        let mut s1 = solved_by_1.clone();
        s1.sort();
        assert_eq!(s1, vec!["RF-001".to_owned(), "RF-002".to_owned()]);

        let mut s2 = solved_by_2.clone();
        s2.sort();
        assert_eq!(s2, vec!["RF-002".to_owned()]);
    }

    #[test]
    fn derive_solved_by_overwrites_stale_data() {
        // SMELL-01 has stale solved_by: [RF-099], but RF-001 actually solves it
        let mut smell = blank_entity("SMELL-01");
        smell
            .relations
            .insert("solved_by".to_owned(), vec!["RF-099".to_owned()]);
        let mut rf = blank_entity("RF-001");
        rf.relations
            .insert("solves".to_owned(), vec!["SMELL-01".to_owned()]);

        let kg = build_graph_from_entities(vec![smell, rf]);

        let solved_by = kg.get_neighbors("SMELL-01", Some("solved_by"));
        assert_eq!(solved_by, vec!["RF-001".to_owned()]);
    }

    #[test]
    fn derive_solved_by_empty_when_no_solves() {
        let smell = blank_entity("SMELL-01");
        let rf = blank_entity("RF-001"); // no solves relation

        let kg = build_graph_from_entities(vec![smell, rf]);
        let solved_by = kg.get_neighbors("SMELL-01", Some("solved_by"));
        assert!(solved_by.is_empty());
    }
}
