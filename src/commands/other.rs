//! Other commands: stats, hooks, telemetry, insight.

use std::collections::HashMap;
use std::io::Read;

use anyhow::{Context, Result};

use episteme::adapters::hooks;
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

/// Generate the next sequential insight ID (TK-xxx).
fn next_insight_id(store: &UserGraphStore) -> String {
    let existing = store.all_user_entity_ids();
    let max_num = existing
        .iter()
        .filter_map(|id| id.strip_prefix("TK-").and_then(|n| n.parse::<u32>().ok()))
        .max()
        .unwrap_or(0);
    format!("TK-{:03}", max_num + 1)
}

fn parse_comma_list(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Produce an ISO-8601 UTC timestamp without depending on chrono.
fn format_timestamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Simple UTC conversion from epoch seconds
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Calculate year, month, day from days since epoch
    let (year, month, day) = days_to_ymd(days_since_epoch);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

/// Convert days since Unix epoch to (year, month, day).
fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    let mut year = 1970u64;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }
    let leap = is_leap(year);
    let month_days: [u64; 12] = if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1u64;
    for &md in &month_days {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }
    (year, month, days + 1)
}

fn is_leap(year: u64) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
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
            let id = next_insight_id(&store);

            let tags = tags.as_deref().map(parse_comma_list).unwrap_or_default();

            let mut relations: HashMap<String, Vec<String>> = HashMap::new();
            if let Some(link_str) = link.as_deref() {
                let linked_ids = parse_comma_list(link_str);
                if !linked_ids.is_empty() {
                    relations.insert("derives_from".to_owned(), linked_ids);
                }
            }

            let now = format_timestamp();
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
            let ids = store.all_user_entity_ids();

            if ids.is_empty() {
                println!("No insights found.");
                return Ok(());
            }

            let display_count = ids.len().min(limit);
            println!("Insights (showing {}/{})", display_count, ids.len());
            println!();
            println!("{:<10} {:<40} {:<10} Created", "ID", "Title", "Tags");
            println!("{}", "-".repeat(80));

            for id in ids.iter().take(limit) {
                if let Some(entity) = store.get_user_entity(id) {
                    let title_display = if entity.title.len() > 38 {
                        format!("{}...", &entity.title[..35])
                    } else {
                        entity.title.clone()
                    };
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
                        id, title_display, tags_display, created
                    );
                }
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
                    let preview = if entity.content.len() > 100 {
                        format!("{}...", &entity.content[..97])
                    } else {
                        entity.content.clone()
                    };
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
