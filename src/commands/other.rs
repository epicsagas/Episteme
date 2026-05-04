//! Other commands: stats, hooks, telemetry.

use std::io::Read;

use anyhow::Result;

use syntagma::adapters::hooks;

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

pub fn cmd_stats() -> Result<()> {
    let graph = load_graph()?;
    let stats = graph.stats();

    println!("Knowledge Graph Statistics");
    println!("==========================");
    println!("Total entities:        {}", stats.total_entities);
    println!("Total edges:           {}", stats.total_edges);
    println!("With relations:        {}", stats.entities_with_relations);
    println!(
        "Avg edges/entity:      {:.2}",
        stats.avg_edges_per_entity
    );
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
            syntagma::adapters::telemetry::write_consent(true).map_err(|e| anyhow::anyhow!(e))?;
            println!("[syntagma] Telemetry enabled.");
            println!("[syntagma] To opt out: syntagma telemetry off");
            Ok(())
        }
        "off" => {
            syntagma::adapters::telemetry::write_consent(false).map_err(|e| anyhow::anyhow!(e))?;
            println!("[syntagma] Telemetry disabled.");
            println!("[syntagma] To re-enable: syntagma telemetry on");
            Ok(())
        }
        "status" => {
            let raw = syntagma::adapters::telemetry::read_consent_raw();
            let state = match raw {
                Some(true) => "enabled",
                Some(false) => "disabled",
                None => "unset (will auto-enable on next command)",
            };
            println!("[syntagma] Telemetry: {state}");
            println!(
                "[syntagma] Consent file: {}",
                syntagma::adapters::paths::syntagma_home()
                    .join("telemetry-consent")
                    .display()
            );
            println!(
                "[syntagma] Install ID:   {}",
                syntagma::adapters::paths::syntagma_home().join("install-id").display()
            );
            Ok(())
        }
        other => anyhow::bail!("unknown telemetry action: {other} (use on|off|status)"),
    }
}
