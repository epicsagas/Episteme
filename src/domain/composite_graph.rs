use std::collections::{HashMap, HashSet, VecDeque};

use crate::domain::graph::KnowledgeGraph;
use crate::domain::types::*;
use crate::ports::graph::{GraphRepository, MutableGraphRepository};

/// Composite graph that merges the immutable canonical knowledge graph
/// with a mutable user knowledge layer.
///
/// Reads are merged from both layers. The canonical graph is never modified.
/// User entities are cached in memory as `Entity` structs for fast access,
/// and persisted via the `MutableGraphRepository` adapter.
pub struct CompositeGraph {
    canonical: KnowledgeGraph,
    user_entities: HashMap<String, Entity>,
    user_reverse: HashMap<String, HashMap<String, Vec<String>>>,
    user_store: Box<dyn MutableGraphRepository>,
}

impl CompositeGraph {
    pub fn new(canonical: KnowledgeGraph, user_store: Box<dyn MutableGraphRepository>) -> Self {
        let mut cg = Self {
            canonical,
            user_entities: HashMap::new(),
            user_reverse: HashMap::new(),
            user_store,
        };
        cg.load_user_entities();
        cg
    }

    pub fn canonical(&self) -> &KnowledgeGraph {
        &self.canonical
    }

    pub fn user_store(&self) -> &dyn MutableGraphRepository {
        self.user_store.as_ref()
    }

    /// Add a user entity to both the in-memory cache and persistent store.
    pub fn add_user_entity(&mut self, entity: UserEntity) -> Result<(), String> {
        let id = entity.id.clone();
        let converted = user_entity_to_entity(&entity);
        self.user_store.add_entity(entity)?;
        self.rebuild_reverse_for(&id, &converted);
        self.user_entities.insert(id, converted);
        Ok(())
    }

    /// Update a user entity.
    pub fn update_user_entity(&mut self, id: &str, entity: UserEntity) -> Result<(), String> {
        let converted = user_entity_to_entity(&entity);
        // Remove old reverse index entries
        self.remove_reverse_for(id);
        self.user_store.update_entity(id, entity)?;
        self.rebuild_reverse_for(id, &converted);
        self.user_entities.insert(id.to_owned(), converted);
        Ok(())
    }

    /// Remove a user entity.
    pub fn remove_user_entity(&mut self, id: &str) -> Result<(), String> {
        self.remove_reverse_for(id);
        self.user_entities.remove(id);
        self.user_store.remove_entity(id)
    }

    /// Add a relation in the user graph and update caches.
    pub fn add_user_relation(
        &mut self,
        from: &str,
        relation: &str,
        to: &str,
    ) -> Result<(), String> {
        self.user_store.add_relation(from, relation, to)?;
        // Update in-memory entity's relations
        if let Some(entity) = self.user_entities.get_mut(from) {
            entity
                .relations
                .entry(relation.to_owned())
                .or_default()
                .push(to.to_owned());
        }
        // Update reverse index
        self.user_reverse
            .entry(to.to_owned())
            .or_default()
            .entry(relation.to_owned())
            .or_default()
            .push(from.to_owned());
        Ok(())
    }

    /// Load all user entities from the persistent store into memory (single query).
    fn load_user_entities(&mut self) {
        for ue in self.user_store.all_user_entities() {
            let id = ue.id.clone();
            let entity = user_entity_to_entity(&ue);
            self.rebuild_reverse_for(&id, &entity);
            self.user_entities.insert(id, entity);
        }
    }

    fn rebuild_reverse_for(&mut self, id: &str, entity: &Entity) {
        for (rel_type, targets) in &entity.relations {
            for target in targets {
                self.user_reverse
                    .entry(target.clone())
                    .or_default()
                    .entry(rel_type.clone())
                    .or_default()
                    .push(id.to_owned());
            }
        }
    }

    fn remove_reverse_for(&mut self, id: &str) {
        if let Some(entity) = self.user_entities.get(id) {
            for (rel_type, targets) in &entity.relations {
                for target in targets {
                    if let Some(rel_map) = self.user_reverse.get_mut(target)
                        && let Some(sources) = rel_map.get_mut(rel_type)
                    {
                        sources.retain(|s| s != id);
                    }
                }
            }
        }
    }

    /// Get user neighbors from in-memory reverse index.
    fn user_incoming_neighbors(&self, entity_id: &str, relation_type: Option<&str>) -> Vec<String> {
        let Some(rel_map) = self.user_reverse.get(entity_id) else {
            return Vec::new();
        };
        let mut neighbors = Vec::new();
        if let Some(rt) = relation_type {
            if let Some(sources) = rel_map.get(rt) {
                neighbors.extend(sources.iter().cloned());
            }
        } else {
            for sources in rel_map.values() {
                neighbors.extend(sources.iter().cloned());
            }
        }
        neighbors
    }

    /// Get user outgoing neighbors from in-memory entity.
    fn user_outgoing_neighbors(&self, entity_id: &str, relation_type: Option<&str>) -> Vec<String> {
        let Some(entity) = self.user_entities.get(entity_id) else {
            return Vec::new();
        };
        let mut neighbors = Vec::new();
        if let Some(rt) = relation_type {
            if let Some(targets) = entity.relations.get(rt) {
                neighbors.extend(targets.iter().cloned());
            }
        } else {
            for targets in entity.relations.values() {
                neighbors.extend(targets.iter().cloned());
            }
        }
        neighbors
    }
}

/// Convert a UserEntity to a canonical Entity for unified graph operations.
fn user_entity_to_entity(ue: &UserEntity) -> Entity {
    Entity {
        id: ue.id.clone(),
        r#type: "insight".to_owned(),
        title: ue.title.clone(),
        description: ue.content.clone(),
        name: ue.title.clone(),
        category: String::new(),
        tags: ue.tags.clone(),
        relations: ue.relations.clone(),
        context: {
            let mut ctx = HashMap::new();
            ctx.insert("author".to_owned(), vec![ue.author.clone()]);
            ctx.insert(
                "confidence".to_owned(),
                vec![format!("{:.2}", ue.confidence)],
            );
            ctx.insert(
                "evidence_count".to_owned(),
                vec![ue.evidence_count.to_string()],
            );
            ctx.insert("last_validated".to_owned(), vec![ue.last_validated.clone()]);
            ctx
        },
        file_path: String::new(),
        source: serde_json::json!({
            "source": "user",
            "confidence": ue.confidence,
            "author": ue.author,
        }),
    }
}

// ---------------------------------------------------------------------------
// GraphRepository implementation — merges canonical + user
// ---------------------------------------------------------------------------

impl GraphRepository for CompositeGraph {
    fn get_entity(&self, id: &str) -> Option<&Entity> {
        self.canonical
            .get_entity(id)
            .or_else(|| self.user_entities.get(id))
    }

    fn get_entities_batch(&self, ids: &[&str]) -> HashMap<String, &Entity> {
        let mut result = self.canonical.get_entities_batch(ids);
        for id in ids {
            if !result.contains_key(*id)
                && let Some(entity) = self.user_entities.get(*id)
            {
                result.insert((*id).to_owned(), entity);
            }
        }
        result
    }

    fn get_neighbors(&self, entity_id: &str, relation_type: Option<&str>) -> Vec<String> {
        let mut canonical = self.canonical.get_neighbors(entity_id, relation_type);
        let user_out = self.user_outgoing_neighbors(entity_id, relation_type);
        let user_in = self.user_incoming_neighbors(entity_id, relation_type);

        canonical.extend(user_out);
        canonical.extend(user_in);
        canonical.sort();
        canonical.dedup();
        canonical
    }

    fn get_all_edges(&self, entity_id: &str) -> Vec<GraphEdge> {
        let mut edges = self.canonical.get_all_edges(entity_id);

        // User outgoing edges
        if let Some(entity) = self.user_entities.get(entity_id) {
            for (rel_type, targets) in &entity.relations {
                for target in targets {
                    edges.push(GraphEdge {
                        from_id: entity_id.to_owned(),
                        relation_type: rel_type.clone(),
                        to_id: target.clone(),
                    });
                }
            }
        }

        // User incoming edges (reverse index)
        if let Some(rel_map) = self.user_reverse.get(entity_id) {
            for (rel_type, sources) in rel_map {
                for source in sources {
                    edges.push(GraphEdge {
                        from_id: source.clone(),
                        relation_type: rel_type.clone(),
                        to_id: entity_id.to_owned(),
                    });
                }
            }
        }

        edges
    }

    fn get_neighborhood(&self, id: &str) -> Option<Neighborhood> {
        let entity = self.get_entity(id)?;

        let mut outgoing: HashMap<String, Vec<String>> = HashMap::new();

        // Canonical outgoing
        if let Some(can_entity) = self.canonical.get_entity(id) {
            for (rt, targets) in &can_entity.relations {
                outgoing
                    .entry(rt.clone())
                    .or_default()
                    .extend(targets.iter().cloned());
            }
        }

        // User outgoing
        if let Some(user_entity) = self.user_entities.get(id) {
            for (rt, targets) in &user_entity.relations {
                outgoing
                    .entry(rt.clone())
                    .or_default()
                    .extend(targets.iter().cloned());
            }
        }

        // Incoming from canonical
        let mut incoming: HashMap<String, Vec<String>> = HashMap::new();
        if let Some(can_nb) = self.canonical.get_neighborhood(id) {
            incoming = can_nb.incoming;
        }

        // Incoming from user reverse index
        if let Some(rel_map) = self.user_reverse.get(id) {
            for (rt, sources) in rel_map {
                incoming
                    .entry(rt.clone())
                    .or_default()
                    .extend(sources.iter().cloned());
            }
        }

        Some(Neighborhood {
            entity: entity.clone(),
            outgoing,
            incoming,
        })
    }

    fn find_shortest_path(
        &self,
        from_id: &str,
        to_id: &str,
        max_depth: usize,
    ) -> Option<Vec<String>> {
        if from_id == to_id && self.get_entity(from_id).is_some() {
            return Some(vec![from_id.to_owned()]);
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        visited.insert(from_id.to_owned());
        queue.push_back(vec![from_id.to_owned()]);

        while let Some(path) = queue.pop_front() {
            if path.len() > max_depth + 1 {
                break;
            }
            let current = path.last()?;

            for neighbor in self.get_neighbors(current, None) {
                if neighbor == to_id {
                    let mut result = path;
                    result.push(neighbor);
                    return Some(result);
                }
                if visited.insert(neighbor.clone()) {
                    let mut new_path = path.clone();
                    new_path.push(neighbor);
                    queue.push_back(new_path);
                }
            }
        }
        None
    }

    fn find_similar_entities(&self, entity_id: &str, threshold: f64) -> Vec<(String, f64)> {
        // Delegate to canonical graph for canonical entities
        let canonical_similar = self.canonical.find_similar_entities(entity_id, threshold);

        // For user entities, compute Jaccard against all entities
        let Some(target) = self.get_entity(entity_id) else {
            return canonical_similar;
        };
        let target_edges: HashSet<String> = target
            .relations
            .iter()
            .flat_map(|(rt, ts)| ts.iter().map(move |t| format!("{rt}:{t}")))
            .collect();

        let mut all_similar = canonical_similar;

        for (other_id, other_entity) in &self.user_entities {
            if other_id == entity_id {
                continue;
            }
            let other_edges: HashSet<String> = other_entity
                .relations
                .iter()
                .flat_map(|(rt, ts)| ts.iter().map(move |t| format!("{rt}:{t}")))
                .collect();

            if target_edges.is_empty() && other_edges.is_empty() {
                continue;
            }

            let intersection = target_edges.intersection(&other_edges).count();
            let union = target_edges.union(&other_edges).count();
            if union > 0 {
                let similarity = intersection as f64 / union as f64;
                if similarity >= threshold {
                    all_similar.push((other_id.clone(), similarity));
                }
            }
        }

        all_similar.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        all_similar
    }

    fn find_contradictions(&self) -> Vec<Contradiction> {
        // Only canonical entities can have contradictions (enforces/violates)
        self.canonical.find_contradictions()
    }

    fn stats(&self) -> GraphStats {
        let mut stats = self.canonical.stats();
        let user_count = self.user_entities.len();
        let mut user_edges = 0usize;
        for entity in self.user_entities.values() {
            for targets in entity.relations.values() {
                user_edges += targets.len();
            }
        }
        stats.total_entities += user_count;
        stats.total_edges += user_edges;
        *stats.by_type.entry("insight".to_owned()).or_insert(0) += user_count;
        stats
    }

    fn all_entity_ids(&self) -> Vec<String> {
        let mut ids = self.canonical.all_entity_ids();
        ids.extend(self.user_entities.keys().cloned());
        ids
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::user_graph_store::UserGraphStore;
    use crate::domain::graph::tests::{blank_entity, build_graph_from_entities};

    fn make_user_entity(id: &str, title: &str) -> UserEntity {
        UserEntity {
            id: id.to_owned(),
            title: title.to_owned(),
            content: format!("Content for {title}"),
            author: "test".to_owned(),
            confidence: 0.5,
            evidence_count: 0,
            last_validated: String::new(),
            tags: vec![],
            relations: HashMap::new(),
            created_at: "2026-01-01T00:00:00Z".to_owned(),
            updated_at: "2026-01-01T00:00:00Z".to_owned(),
        }
    }

    fn build_composite(canonical_entities: Vec<Entity>) -> CompositeGraph {
        let kg = build_graph_from_entities(canonical_entities);
        let store = UserGraphStore::open_in_memory().unwrap();
        CompositeGraph::new(kg, Box::new(store))
    }

    #[test]
    fn get_entity_checks_canonical_first() {
        let mut e = blank_entity("DP-001");
        e.title = "Singleton".to_owned();
        let cg = build_composite(vec![e]);
        let found = cg.get_entity("DP-001").unwrap();
        assert_eq!(found.title, "Singleton");
    }

    #[test]
    fn get_entity_falls_back_to_user() {
        let mut cg = build_composite(vec![]);
        cg.add_user_entity(make_user_entity("TK-001", "Team Decision"))
            .unwrap();
        let found = cg.get_entity("TK-001").unwrap();
        assert_eq!(found.title, "Team Decision");
        assert_eq!(found.r#type, "insight");
    }

    #[test]
    fn get_neighbors_merges_canonical_and_user() {
        let mut smell = blank_entity("SMELL-01");
        smell
            .relations
            .insert("solved_by".to_owned(), vec!["RF-001".to_owned()]);
        let rf = blank_entity("RF-001");

        let mut cg = build_composite(vec![smell, rf]);

        let mut ue = make_user_entity("TK-001", "Insight");
        ue.relations
            .insert("derives_from".to_owned(), vec!["SMELL-01".to_owned()]);
        cg.add_user_entity(ue).unwrap();

        let neighbors = cg.get_neighbors("SMELL-01", None);
        assert!(neighbors.contains(&"RF-001".to_owned()));
        assert!(neighbors.contains(&"TK-001".to_owned()));
    }

    #[test]
    fn find_path_crosses_both_layers() {
        let mut dp = blank_entity("DP-005");
        dp.relations
            .insert("solves".to_owned(), vec!["SMELL-01".to_owned()]);
        let smell = blank_entity("SMELL-01");

        let mut cg = build_composite(vec![dp, smell]);

        let mut ue = make_user_entity("TK-001", "Insight");
        ue.relations
            .insert("derives_from".to_owned(), vec!["DP-005".to_owned()]);
        cg.add_user_entity(ue).unwrap();

        // Path from user entity through canonical to smell
        let path = cg.find_shortest_path("TK-001", "SMELL-01", 5);
        assert!(path.is_some(), "should find path across layers");
        let p = path.unwrap();
        assert_eq!(p.first().unwrap(), "TK-001");
        assert_eq!(p.last().unwrap(), "SMELL-01");
    }

    #[test]
    fn stats_merges_counts() {
        let mut cg = build_composite(vec![blank_entity("DP-001"), blank_entity("RF-001")]);

        let mut ue = make_user_entity("TK-001", "Insight");
        ue.relations
            .insert("derives_from".to_owned(), vec!["DP-001".to_owned()]);
        cg.add_user_entity(ue).unwrap();

        let stats = cg.stats();
        assert_eq!(stats.total_entities, 3);
        assert_eq!(*stats.by_type.get("insight").unwrap_or(&0), 1);
    }

    #[test]
    fn remove_user_entity() {
        let mut cg = build_composite(vec![]);
        cg.add_user_entity(make_user_entity("TK-001", "To Remove"))
            .unwrap();
        assert!(cg.get_entity("TK-001").is_some());

        cg.remove_user_entity("TK-001").unwrap();
        assert!(cg.get_entity("TK-001").is_none());
    }

    #[test]
    fn canonical_untouched_by_user_operations() {
        let mut dp = blank_entity("DP-001");
        dp.title = "Original".to_owned();
        dp.relations
            .insert("solves".to_owned(), vec!["SMELL-01".to_owned()]);

        let mut cg = build_composite(vec![dp]);

        let mut ue = make_user_entity("TK-001", "Insight");
        ue.relations
            .insert("derives_from".to_owned(), vec!["DP-001".to_owned()]);
        cg.add_user_entity(ue).unwrap();
        cg.remove_user_entity("TK-001").unwrap();

        // Canonical should be completely unchanged
        let canonical = cg.canonical();
        let dp = canonical.get_entity("DP-001").unwrap();
        assert_eq!(dp.title, "Original");
        let can_neighbors = canonical.get_neighbors("DP-001", Some("solves"));
        assert_eq!(can_neighbors, vec!["SMELL-01"]);
    }

    #[test]
    fn graph_repository_trait_is_implemented() {
        fn assert_graph_repo<T: GraphRepository>(_: &T) {}
        let cg = build_composite(vec![]);
        assert_graph_repo(&cg);
    }
}
