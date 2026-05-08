//! Explore command: keyword search and interactive REPL.

use anyhow::Result;

use syntagma_engine::server::mcp_handler::SyntagmaMCP;

use super::prelude::*;

pub fn cmd_explore(
    query: Option<String>,
    limit: usize,
    entity_type: Option<&str>,
    interactive: bool,
) -> Result<()> {
    // Enter interactive REPL when requested or no query provided.
    if interactive || query.is_none() {
        return cmd_explore_repl(limit, entity_type);
    }

    let query = query.unwrap();
    let graph = load_graph()?;

    // Use the MCP server's keyword search to find matching entities.
    // Attach the RAG database (FTS5 + embeddings) when available for
    // higher-quality hybrid search results.
    let mut mcp = SyntagmaMCP::new(graph);
    mcp.try_attach_rag();
    let result = mcp.search_knowledge(&query, Some(limit), entity_type);

    let results = result
        .get("results")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let _count = result.get("count").and_then(as_u64_or_zero).unwrap_or(0);

    if results.is_empty() {
        println!("No results found for '{}'.", query);
        return Ok(());
    }

    // Deduplicate by entity_id (RAG returns chunk-level results).
    let mut seen_ids = std::collections::HashSet::new();
    let deduped: Vec<_> = results
        .iter()
        .filter(|e| {
            let id = e.get("entity_id").and_then(|v| v.as_str()).unwrap_or("?");
            seen_ids.insert(id.to_owned())
        })
        .collect();

    println!("Found {} result(s) for '{}':", deduped.len(), query);
    println!();
    for entry in &deduped {
        let id = entry
            .get("entity_id")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let title = entry.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let etype = entry.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let category = entry.get("category").and_then(|v| v.as_str()).unwrap_or("");
        // Score may be an integer (keyword fallback) or a float string (RAG hybrid).
        let score_display = match entry.get("score") {
            Some(v) if v.is_u64() => v.as_u64().unwrap_or(0).to_string(),
            Some(v) if v.is_f64() => format!("{:.4}", v.as_f64().unwrap_or(0.0)),
            Some(v) => v.as_str().unwrap_or("0").to_owned(),
            None => "0".to_owned(),
        };

        println!("  [{}] {} ({})", id, title, etype);
        if !category.is_empty() {
            println!("    Category: {}", category);
        }
        println!("    Relevance score: {}", score_display);
        println!();
    }

    Ok(())
}

/// Interactive REPL for exploring the knowledge graph.
fn cmd_explore_repl(limit: usize, entity_type: Option<&str>) -> Result<()> {
    let graph = load_graph()?;
    let mut mcp = SyntagmaMCP::new(graph);
    mcp.try_attach_rag();
    let mut rl = rustyline::DefaultEditor::new()?;

    println!("syntagma interactive explorer");
    println!("Commands: search <query>, entity <id>, neighbors <id>, path <from> <to>,");
    println!("          stats, contradictions, quit/exit");
    println!();

    loop {
        let line = match rl.readline("syntagma> ") {
            Ok(line) => line,
            Err(rustyline::error::ReadlineError::Eof) => break,
            Err(rustyline::error::ReadlineError::Interrupted) => break,
            Err(e) => return Err(e.into()),
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "quit" || trimmed == "exit" {
            break;
        }

        let _ = rl.add_history_entry(&line);

        let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
        let cmd = parts[0];
        let arg = parts.get(1).copied().unwrap_or("");

        match cmd {
            "search" => {
                if arg.is_empty() {
                    println!("Usage: search <query>");
                    continue;
                }
                let result = mcp.search_knowledge(arg, Some(limit), entity_type);
                print_search_results(arg, &result);
            }
            "entity" => {
                if arg.is_empty() {
                    println!("Usage: entity <id>");
                    continue;
                }
                let result = mcp.get_entity(arg, Some("summary"));
                if let Some(err) = result.get("error") {
                    println!("Error: {}", err.as_str().unwrap_or("unknown"));
                } else {
                    println!("{}", serde_json::to_string_pretty(&result)?);
                }
            }
            "neighbors" => {
                if arg.is_empty() {
                    println!("Usage: neighbors <id>");
                    continue;
                }
                let result = mcp.get_neighbors(arg, None);
                if let Some(err) = result.get("error") {
                    println!("Error: {}", err.as_str().unwrap_or("unknown"));
                } else {
                    let neighbors = result
                        .get("neighbors")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();
                    if neighbors.is_empty() {
                        println!("No neighbors found for '{}'.", arg);
                    } else {
                        for n in &neighbors {
                            let nid = n.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                            let title = n.get("title").and_then(|v| v.as_str()).unwrap_or("");
                            let ntype = n.get("type").and_then(|v| v.as_str()).unwrap_or("");
                            println!("  [{}] {} ({})", nid, title, ntype);
                        }
                    }
                }
            }
            "path" => {
                let path_args: Vec<&str> = arg.split_whitespace().collect();
                if path_args.len() < 2 {
                    println!("Usage: path <from_id> <to_id>");
                    continue;
                }
                let result = mcp.find_path(path_args[0], path_args[1], None);
                if let Some(err) = result.get("error") {
                    println!("Error: {}", err.as_str().unwrap_or("unknown"));
                } else {
                    let path = result
                        .get("path")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();
                    let length = result.get("length").and_then(|v| v.as_u64()).unwrap_or(0);
                    println!("Path ({} hops):", length);
                    for (i, p) in path.iter().enumerate() {
                        let pid = p.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                        let title = p.get("title").and_then(|v| v.as_str()).unwrap_or("");
                        let arrow = if i == 0 { "" } else { " -> " };
                        print!("{}[{}] {}", arrow, pid, title);
                    }
                    println!();
                }
            }
            "stats" => {
                let result = mcp.handle_resource_read("syntagma://stats");
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
            "contradictions" => {
                let result = mcp.handle_resource_read("syntagma://contradictions");
                let contradictions = result.as_array();
                match contradictions {
                    Some(c) if c.is_empty() => println!("No contradictions found."),
                    Some(c) => {
                        println!("{} contradiction(s):", c.len());
                        for entry in c {
                            let eid = entry
                                .get("entity_id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("?");
                            let title = entry.get("title").and_then(|v| v.as_str()).unwrap_or("");
                            println!("  [{}] {}", eid, title);
                        }
                    }
                    None => println!("{}", serde_json::to_string_pretty(&result)?),
                }
            }
            _ => {
                println!(
                    "Unknown command '{}'. Try: search, entity, neighbors, path, stats, contradictions",
                    cmd
                );
            }
        }
    }

    Ok(())
}

/// Print search results from a `search_knowledge` response value.
pub fn print_search_results(query: &str, result: &serde_json::Value) {
    let results = result
        .get("results")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let _count = result.get("count").and_then(as_u64_or_zero).unwrap_or(0);

    if results.is_empty() {
        println!("No results found for '{}'.", query);
        return;
    }

    // Deduplicate by entity_id (RAG returns chunk-level results).
    let mut seen_ids = std::collections::HashSet::new();
    let deduped: Vec<_> = results
        .iter()
        .filter(|e| {
            let id = e.get("entity_id").and_then(|v| v.as_str()).unwrap_or("?");
            seen_ids.insert(id.to_owned())
        })
        .collect();

    println!("Found {} result(s) for '{}':", deduped.len(), query);
    for entry in &deduped {
        let id = entry
            .get("entity_id")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let title = entry.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let etype = entry.get("type").and_then(|v| v.as_str()).unwrap_or("");
        println!("  [{}] {} ({})", id, title, etype);
    }
}
