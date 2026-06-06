//! Other commands: stats, hooks, telemetry, insight.

use std::collections::HashMap;
use std::io::Read;

use anyhow::{Context, Result};

use episteme::adapters::hooks;
use episteme::adapters::insight_utils;
use episteme::adapters::paths;
use episteme::adapters::user_graph_store::UserGraphStore;
use episteme::domain::types::UserEntity;
use episteme::ports::graph::MutableGraphRepository;

use super::prelude::*;

/// Dispatch type for hooks subcommands, avoiding direct clap enum coupling.
pub enum HooksOp {
    Ground {
        prompt: Option<String>,
        limit: usize,
        json: bool,
    },
    Sniff {
        files: Vec<String>,
        staged: bool,
        min_confidence: f64,
        _json: bool,
        verbose: bool,
    },
    Audit {
        file: Option<String>,
        _json: bool,
    },
}

/// Dispatch type for insight subcommands.
pub enum InsightOp {
    Add {
        title: String,
        content: String,
        tags: Option<String>,
        link: Option<String>,
    },
    List {
        limit: usize,
    },
    Search {
        query: String,
        limit: usize,
    },
}

// ---------------------------------------------------------------------------
// Insight command handler
// ---------------------------------------------------------------------------

fn open_user_store() -> Result<UserGraphStore> {
    let db_path = paths::episteme_home().join("user_knowledge.db");
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory {}", parent.display()))?;
    }
    UserGraphStore::open(&db_path).map_err(|e| anyhow::anyhow!(e))
}

pub fn cmd_insight(op: InsightOp) -> Result<()> {
    match op {
        InsightOp::Add {
            title,
            content,
            tags,
            link,
        } => {
            let store = open_user_store()?;
            let id =
                insight_utils::next_insight_id_atomic(&store).map_err(|e| anyhow::anyhow!(e))?;

            let tags = tags
                .as_deref()
                .map(insight_utils::parse_comma_list)
                .unwrap_or_default();

            let mut relations: HashMap<String, Vec<String>> = HashMap::new();
            if let Some(link_str) = link.as_deref() {
                let linked_ids = insight_utils::parse_comma_list(link_str);
                if !linked_ids.is_empty() {
                    relations.insert("derives_from".to_owned(), linked_ids);
                }
            }

            let now = insight_utils::format_timestamp();
            let entity = UserEntity {
                id: id.clone(),
                title,
                content,
                author: "user".to_owned(),
                confidence: 0.5,
                evidence_count: 0,
                last_validated: String::new(),
                tags,
                relations,
                link_provenance: std::collections::HashMap::new(),
                created_at: now.clone(),
                updated_at: now,
            };

            store.add_entity(entity).map_err(|e| anyhow::anyhow!(e))?;

            println!("Added insight [{}]", id);
            println!(
                "  Title: {}",
                store
                    .get_user_entity(&id)
                    .map(|e| e.title)
                    .unwrap_or_default()
            );

            let count = store.user_entity_count();
            println!("  Total insights: {}", count);
            Ok(())
        }
        InsightOp::List { limit } => {
            let store = open_user_store()?;
            let entities = store.all_user_entities();

            if entities.is_empty() {
                println!("No insights found.");
                return Ok(());
            }

            let display_count = entities.len().min(limit);
            println!("Insights (showing {}/{})", display_count, entities.len());
            println!();
            println!("{:<10} {:<40} {:<10} Created", "ID", "Title", "Tags");
            println!("{}", "-".repeat(80));

            for entity in entities.iter().take(limit) {
                let title_display = insight_utils::truncate_text(&entity.title, 38);
                let tags_display = if entity.tags.is_empty() {
                    "-".to_owned()
                } else {
                    entity.tags.join(",")
                };
                let created = if entity.created_at.len() >= 10 {
                    &entity.created_at[..10]
                } else {
                    &entity.created_at
                };
                println!(
                    "{:<10} {:<40} {:<10} {}",
                    entity.id, title_display, tags_display, created
                );
            }

            Ok(())
        }
        InsightOp::Search { query, limit } => {
            let store = open_user_store()?;
            let results = store.search_user_entities(&query, limit);

            if results.is_empty() {
                println!("No insights matching '{}'.", query);
                return Ok(());
            }

            println!("Search results for '{}' ({} found):", query, results.len());
            println!();
            for entity in &results {
                println!("  [{}] {}", entity.id, entity.title);
                if !entity.content.is_empty() {
                    let preview = insight_utils::truncate_text(&entity.content, 100);
                    println!("    {}", preview);
                }
                if !entity.tags.is_empty() {
                    println!("    Tags: {}", entity.tags.join(", "));
                }
                let linked: Vec<&String> = entity.relations.values().flatten().collect();
                if !linked.is_empty() {
                    println!(
                        "    Linked: {}",
                        linked
                            .iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
                println!();
            }

            Ok(())
        }
    }
}

pub fn cmd_stats() -> Result<()> {
    let graph = load_graph()?;
    let stats = graph.stats();

    println!("Knowledge Graph Statistics");
    println!("==========================");
    println!("Total entities:        {}", stats.total_entities);
    println!("Total edges:           {}", stats.total_edges);
    println!("With relations:        {}", stats.entities_with_relations);
    println!("Avg edges/entity:      {:.2}", stats.avg_edges_per_entity);
    println!();
    println!("By type:");
    let mut types: Vec<_> = stats.by_type.iter().collect();
    types.sort_by(|a, b| b.1.cmp(a.1));
    for (t, count) in &types {
        println!("  {:20} {}", t, count);
    }

    // Embedding model info from RAG database.
    let db_path = episteme::adapters::paths::db_path();
    if db_path.exists() {
        use episteme::adapters::config::EpistemeConfig;
        use episteme::adapters::infra::sqlite_db;

        if let Ok(conn) = rusqlite::Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        ) {
            let stored_model = sqlite_db::get_meta(&conn, "embedding_model").ok().flatten();
            let stored_dim = sqlite_db::get_meta(&conn, "embedding_dim")
                .ok()
                .flatten()
                .and_then(|v| v.parse::<usize>().ok());

            let cfg = EpistemeConfig::load().unwrap_or_default();
            let configured = match cfg.embedding_provider.to_lowercase().as_str() {
                "openai" => format!("openai:{}", cfg.openai_embed_model),
                _ => cfg.embedding_model.clone(),
            };

            println!();
            println!("Embedding:");
            match (&stored_model, &stored_dim) {
                (Some(m), Some(d)) => {
                    println!("  DB model:           {} ({}-dim)", m, d);
                }
                (Some(m), None) => {
                    println!("  DB model:           {}", m);
                }
                _ => {
                    println!("  DB model:           (not recorded)");
                }
            }
            println!("  Configured now:     {}", configured);

            if let Some(stored) = &stored_model
                && stored != &configured
            {
                println!(
                    "  ⚠  WARNING: model mismatch — DB was built with '{}', current config is '{}'",
                    stored, configured
                );
            }
        }
    }

    Ok(())
}

pub fn cmd_hooks(sub: HooksOp) -> Result<()> {
    match sub {
        HooksOp::Ground {
            prompt,
            limit,
            json,
        } => {
            let graph = load_graph()?;
            let prompt = prompt.unwrap_or_else(|| {
                // Read from stdin if no prompt given.
                let mut buf = String::new();
                let _ = std::io::stdin().read_to_string(&mut buf);
                buf
            });
            let output = hooks::handle_ground(&graph, &prompt, limit);
            if json {
                println!("{}", serde_json::json!({"ground": output}));
            } else {
                print!("{output}");
            }
            Ok(())
        }
        HooksOp::Sniff {
            mut files,
            staged,
            min_confidence,
            _json,
            verbose,
        } => {
            if staged {
                files.extend(hooks::get_staged_files());
            }
            if files.is_empty() {
                println!("No files to sniff. Provide file paths or --staged.");
                return Ok(());
            }
            if verbose {
                eprintln!("Sniffing {} file(s)...", files.len());
            }
            let output = hooks::handle_sniff(&files, min_confidence);
            print!("{output}");
            Ok(())
        }
        HooksOp::Audit { file, _json } => {
            let output = hooks::handle_audit(file.as_deref(), 0.5);
            print!("{output}");
            Ok(())
        }
    }
}

pub fn cmd_telemetry(action: &str) -> Result<()> {
    match action.trim().to_lowercase().as_str() {
        "on" => {
            episteme::adapters::telemetry::write_consent(true).map_err(|e| anyhow::anyhow!(e))?;
            println!("[episteme] Telemetry enabled.");
            println!("[episteme] To opt out: epis telemetry off");
            Ok(())
        }
        "off" => {
            episteme::adapters::telemetry::write_consent(false).map_err(|e| anyhow::anyhow!(e))?;
            println!("[episteme] Telemetry disabled.");
            println!("[episteme] To re-enable: epis telemetry on");
            Ok(())
        }
        "status" => {
            let raw = episteme::adapters::telemetry::read_consent_raw();
            let state = match raw {
                Some(true) => "enabled",
                Some(false) => "disabled",
                None => "unset (will auto-enable on next command)",
            };
            println!("[episteme] Telemetry: {state}");
            println!(
                "[episteme] Consent file: {}",
                episteme::adapters::paths::episteme_home()
                    .join("telemetry-consent")
                    .display()
            );
            println!(
                "[episteme] Install ID:   {}",
                episteme::adapters::paths::episteme_home()
                    .join("install-id")
                    .display()
            );
            Ok(())
        }
        other => anyhow::bail!("unknown telemetry action: {other} (use on|off|status)"),
    }
}
