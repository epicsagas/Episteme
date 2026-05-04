//! Graph command: query the knowledge graph structure.

use anyhow::Result;

use super::prelude::*;

/// Public trait-like enum so graph commands can be dispatched without
/// importing the CLI enum directly. The main.rs match converts `GraphCommands`
/// (the clap enum) into this one.
pub enum GraphOp {
    Entity { id: String },
    Neighbors { id: String, relation_type: Option<String> },
    Path { from: String, to: String, max_depth: usize },
    Contradictions,
}

pub fn cmd_graph(op: GraphOp) -> Result<()> {
    // Load the graph once, then dispatch.
    let graph = load_graph()?;

    match op {
        GraphOp::Entity { id } => {
            match graph.get_entity(&id) {
                Some(entity) => {
                    let json = serde_json::to_string_pretty(&entity)?;
                    println!("{}", json);
                }
                None => {
                    anyhow::bail!("entity '{}' not found", id);
                }
            }
        }
        GraphOp::Neighbors {
            id,
            relation_type,
        } => {
            let neighbors = graph.get_neighbors(&id, relation_type.as_deref());

            if neighbors.is_empty() {
                println!("No neighbors found for '{}'.", id);
                return Ok(());
            }

            // Resolve entity details for each neighbor.
            let ids_ref: Vec<&str> = neighbors.iter().map(|s| s.as_str()).collect();
            let batch = graph.get_entities_batch(&ids_ref);

            println!("Neighbors of {} ({} found):", id, neighbors.len());
            println!();
            for nid in &neighbors {
                if let Some(e) = batch.get(nid.as_str()) {
                    println!(
                        "  [{}] {} ({})",
                        nid,
                        if e.title.is_empty() {
                            &e.name
                        } else {
                            &e.title
                        },
                        e.r#type
                    );
                } else {
                    println!("  [{}] (unknown)", nid);
                }
            }
        }
        GraphOp::Path {
            from,
            to,
            max_depth,
        } => {
            match graph.find_shortest_path(&from, &to, max_depth) {
                Some(path) => {
                    // Resolve entity names for the path.
                    let ids_ref: Vec<&str> = path.iter().map(|s| s.as_str()).collect();
                    let batch = graph.get_entities_batch(&ids_ref);

                    println!(
                        "Path from {} to {} ({} hops):",
                        from,
                        to,
                        path.len().saturating_sub(1)
                    );
                    for (i, pid) in path.iter().enumerate() {
                        let title = batch
                            .get(pid.as_str())
                            .map(|e| {
                                if e.title.is_empty() {
                                    e.name.clone()
                                } else {
                                    e.title.clone()
                                }
                            })
                            .unwrap_or_default();
                        let arrow = if i == 0 { "" } else { " -> " };
                        print!("{}[{}] {}", arrow, pid, title);
                    }
                    println!();
                }
                None => {
                    println!(
                        "No path found between '{}' and '{}' within {} hops.",
                        from, to, max_depth
                    );
                }
            }
        }
        GraphOp::Contradictions => {
            let contradictions = graph.find_contradictions();

            if contradictions.is_empty() {
                println!("No contradictions found in the knowledge graph.");
                return Ok(());
            }

            println!("Found {} contradiction(s):", contradictions.len());
            println!();
            for c in &contradictions {
                println!("  [{}] {}", c.entity_id, c.title);
                println!("    Conflicts: {}", c.conflicts.join(", "));
                println!();
            }
        }
    }
    Ok(())
}
