//! Standalone MCP stdio server binary for Syntagma.
//!
//! Reads JSON-RPC requests from stdin and writes responses to stdout.

use std::io::{self, BufRead, Write};

use anyhow::{Context, Result};

fn main() -> Result<()> {
    // Install a minimal tracing subscriber so library log messages don't panic.
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .try_init();

    // Load the knowledge graph from the default data directory.
    let data_dir = syntagma::paths::data_dir();
    let graph = syntagma::adapters::json_loader::load_graph(&data_dir).with_context(|| {
        format!(
            "failed to load knowledge graph from {}",
            data_dir.display()
        )
    })?;

    let mut mcp = syntagma::server::mcp_handler::SyntagmaMCP::new(graph);
    mcp.try_attach_rag();

    eprintln!("syntagma MCP server: reading JSON-RPC from stdin");

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line.context("failed to read from stdin")?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let request: serde_json::Value =
            serde_json::from_str(trimmed).with_context(|| "invalid JSON on stdin")?;

        if let Some(response) = syntagma::server::mcp_dispatcher::dispatch(&mcp, request) {
            let response_str = serde_json::to_string(&response)?;
            writeln!(stdout_lock, "{}", response_str)
                .context("failed to write response")?;
            stdout_lock.flush().context("failed to flush stdout")?;
        }
    }

    Ok(())
}
