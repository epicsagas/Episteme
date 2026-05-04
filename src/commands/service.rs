//! Service commands: service, api, mcp, mcp_http, web.

use std::net::SocketAddr;

use anyhow::{Context, Result};

use syntagma::adapters::config::SyntagmaConfig;
use syntagma::server::mcp_handler::SyntagmaMCP;

use super::prelude::*;

/// Dispatch type for service subcommands, avoiding direct clap enum coupling.
pub enum ServiceOp {
    Serve { host: String, port: u16 },
    Start { host: String, port: u16 },
    Stop,
    Restart { host: String, port: u16 },
    Status,
    LaunchdInstall { host: String, port: u16 },
    LaunchdUninstall,
    LaunchdStatus,
    Enable { now: bool },
    Disable { now: bool },
}

pub fn cmd_service(sub: ServiceOp) -> Result<()> {
    match sub {
        ServiceOp::Serve { host, port } => cmd_mcp(true, &host, port),
        ServiceOp::Start { host, port } => {
            let pid = syntagma::adapters::service::cmd_start(&host, port)
                .map_err(|e| anyhow::anyhow!(e))?;
            println!("MCP server started (PID {pid})");
            Ok(())
        }
        ServiceOp::Stop => {
            syntagma::adapters::service::cmd_stop().map_err(|e| anyhow::anyhow!(e))?;
            println!("MCP server stopped");
            Ok(())
        }
        ServiceOp::Restart { host, port } => {
            // Best-effort stop; ignore errors if nothing was running.
            let _ = syntagma::adapters::service::cmd_stop();
            let pid = syntagma::adapters::service::cmd_start(&host, port)
                .map_err(|e| anyhow::anyhow!(e))?;
            println!("MCP server restarted (PID {pid})");
            Ok(())
        }
        ServiceOp::Status => {
            syntagma::adapters::service::cmd_status();
            Ok(())
        }
        ServiceOp::LaunchdInstall { host, port } => {
            let msg = syntagma::adapters::service::install_launchd_agent(&host, port)
                .map_err(|e| anyhow::anyhow!(e))?;
            println!("{msg}");
            Ok(())
        }
        ServiceOp::LaunchdUninstall => {
            let msg = syntagma::adapters::service::uninstall_launchd_agent()
                .map_err(|e| anyhow::anyhow!(e))?;
            println!("{msg}");
            Ok(())
        }
        ServiceOp::LaunchdStatus => {
            let msg = syntagma::adapters::service::launchd_status()
                .map_err(|e| anyhow::anyhow!(e))?;
            println!("{msg}");
            Ok(())
        }
        ServiceOp::Enable { now } => {
            let msg = syntagma::adapters::service::enable_launchd(now)
                .map_err(|e| anyhow::anyhow!(e))?;
            println!("{msg}");
            Ok(())
        }
        ServiceOp::Disable { now } => {
            let msg = syntagma::adapters::service::disable_launchd(now)
                .map_err(|e| anyhow::anyhow!(e))?;
            println!("{msg}");
            Ok(())
        }
    }
}

pub fn cmd_api(host: &str, port: u16) -> Result<()> {
    let config = SyntagmaConfig::load()?;
    let mut config = config;
    config.api_host = host.to_owned();
    config.api_port = port;

    let graph = load_graph()?;

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async { syntagma::server::api_server::run(&config, graph).await })
}

pub fn cmd_mcp(http: bool, host: &str, port: u16) -> Result<()> {
    if http {
        return cmd_mcp_http(host, port);
    }

    let graph = load_graph()?;
    let mut mcp = SyntagmaMCP::new(graph);
    mcp.try_attach_rag();

    eprintln!("syntagma MCP server (stdio transport)");
    eprintln!("Reading JSON-RPC requests from stdin, writing responses to stdout...");

    use std::io::{self, BufRead, Write};

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
            serde_json::from_str(trimmed).with_context(|| "invalid JSON")?;

        if let Some(response) = syntagma::server::mcp_dispatcher::dispatch(&mcp, request) {
            let response_str = serde_json::to_string(&response)?;
            writeln!(stdout_lock, "{}", response_str)
                        .context("failed to write response")?;
            stdout_lock.flush().context("failed to flush stdout")?;
        }
    }

    Ok(())
}

fn cmd_mcp_http(host: &str, port: u16) -> Result<()> {
    let cfg = SyntagmaConfig::load()?;
    let allowed_api_keys = syntagma::server::mcp_auth::parse_api_keys(&cfg.api_keys);
    let graph = load_graph()?;
    let mut mcp = SyntagmaMCP::new(graph);
    mcp.try_attach_rag();

    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .with_context(|| format!("invalid bind address: {host}:{port}"))?;
    let app = syntagma::server::mcp_transport_http::mcp_http_router(mcp, allowed_api_keys);

    println!("syntagma MCP server (HTTP): http://{host}:{port}/mcp");
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .with_context(|| format!("failed to bind MCP HTTP listener on {addr}"))?;
        axum::serve(listener, app)
            .await
            .context("MCP HTTP server failed")
    })
}

pub fn cmd_web(host: &str, port: u16) -> Result<()> {
    let graph = load_graph()?;
    let handler = std::sync::Arc::new(SyntagmaMCP::new(graph));
    let app = syntagma::server::web_viewer::web_router(handler);

    let addr = format!("{host}:{port}");

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        println!("Graph viewer at http://{addr}");
        axum::serve(listener, app).await?;
        Ok(())
    })
}
