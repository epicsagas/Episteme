//! Service commands: service, api, mcp, mcp_http, web.

use std::net::SocketAddr;

use anyhow::{Context, Result};

use episteme::adapters::config::EpistemeConfig;
use episteme::adapters::paths;
use episteme::adapters::service::ServiceKind;
use episteme::adapters::user_graph_store::UserGraphStore;
use episteme::domain::composite_graph::CompositeGraph;
use episteme::server::mcp_handler::EpistemeMCP;

use super::prelude::*;

/// Dispatch type for service subcommands, avoiding direct clap enum coupling.
pub enum ServiceOp {
    Serve {
        host: String,
        port: u16,
        kind: ServiceKind,
    },
    Start {
        host: String,
        port: u16,
        kind: ServiceKind,
    },
    Stop {
        kind: ServiceKind,
    },
    Restart {
        host: String,
        port: u16,
        kind: ServiceKind,
    },
    Status {
        kind: ServiceKind,
    },
    LaunchdInstall {
        host: String,
        port: u16,
    },
    LaunchdUninstall,
    LaunchdStatus,
    Enable {
        now: bool,
        kind: ServiceKind,
    },
    Disable {
        now: bool,
        kind: ServiceKind,
    },
}

pub fn cmd_service(sub: ServiceOp) -> Result<()> {
    match sub {
        ServiceOp::Serve { host, port, kind } => match kind {
            ServiceKind::Mcp => cmd_mcp(true, &host, port),
            ServiceKind::Api => cmd_api(&host, port),
        },
        ServiceOp::Start { host, port, kind } => {
            let label = kind_label(kind);
            let pid = episteme::adapters::service::cmd_start(kind, &host, port)
                .map_err(|e| anyhow::anyhow!(e))?;
            if pid == 0 {
                println!("{label} server is already running");
            } else {
                println!("{label} server started (PID {pid})");
            }
            Ok(())
        }
        ServiceOp::Stop { kind } => {
            let label = kind_label(kind);
            episteme::adapters::service::cmd_stop(kind).map_err(|e| anyhow::anyhow!(e))?;
            println!("{label} server stopped");
            Ok(())
        }
        ServiceOp::Restart { host, port, kind } => {
            let label = kind_label(kind);
            // Stop first — handles launchd unload and process kill.
            let _ = episteme::adapters::service::cmd_stop(kind);
            let pid = episteme::adapters::service::cmd_start(kind, &host, port)
                .map_err(|e| anyhow::anyhow!(e))?;
            if pid == 0 {
                println!("{label} server is already running");
            } else {
                println!("{label} server restarted (PID {pid})");
            }
            Ok(())
        }
        ServiceOp::Status { kind } => {
            episteme::adapters::service::cmd_status(kind);
            Ok(())
        }
        ServiceOp::LaunchdInstall { host, port } => {
            let msg = episteme::adapters::service::install_launchd_agent(&host, port)
                .map_err(|e| anyhow::anyhow!(e))?;
            println!("{msg}");
            Ok(())
        }
        ServiceOp::LaunchdUninstall => {
            let msg = episteme::adapters::service::uninstall_launchd_agent()
                .map_err(|e| anyhow::anyhow!(e))?;
            println!("{msg}");
            Ok(())
        }
        ServiceOp::LaunchdStatus => {
            let msg =
                episteme::adapters::service::launchd_status().map_err(|e| anyhow::anyhow!(e))?;
            println!("{msg}");
            Ok(())
        }
        ServiceOp::Enable { now, kind } => {
            let msg = episteme::adapters::service::enable_service(kind, now)
                .map_err(|e| anyhow::anyhow!(e))?;
            println!("{msg}");
            Ok(())
        }
        ServiceOp::Disable { now, kind } => {
            let msg = episteme::adapters::service::disable_service(kind, now)
                .map_err(|e| anyhow::anyhow!(e))?;
            println!("{msg}");
            Ok(())
        }
    }
}

fn kind_label(kind: ServiceKind) -> &'static str {
    episteme::adapters::service::kind_label(kind)
}

pub fn cmd_api(host: &str, port: u16) -> Result<()> {
    let config = EpistemeConfig::load()?;
    let mut config = config;
    config.api_host = host.to_owned();
    config.api_port = port;

    let graph = load_graph()?;

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async { episteme::server::api_server::run(&config, graph).await })
}

pub fn cmd_mcp(http: bool, host: &str, port: u16) -> Result<()> {
    if http {
        return cmd_mcp_http(host, port);
    }

    let graph = load_graph()?;
    let mut mcp = build_mcp(graph);
    mcp.try_attach_rag();

    eprintln!("episteme MCP server (stdio transport)");
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

        if let Some(response) = episteme::server::mcp_dispatcher::dispatch(&mcp, request) {
            let response_str = serde_json::to_string(&response)?;
            writeln!(stdout_lock, "{}", response_str).context("failed to write response")?;
            stdout_lock.flush().context("failed to flush stdout")?;
        }
    }

    Ok(())
}

fn cmd_mcp_http(host: &str, port: u16) -> Result<()> {
    let cfg = EpistemeConfig::load()?;

    // Security: refuse non-localhost binding without a bearer token.
    if !episteme::server::mcp_auth::is_localhost(host) && cfg.mcp_token.is_empty() {
        anyhow::bail!(
            "Refusing to bind on {host} without a bearer token.\n\
             Run 'epis install' to configure a token, or set EPISTEME_MCP_TOKEN."
        );
    }

    let mut allowed_api_keys = episteme::server::mcp_auth::parse_api_keys(&cfg.api_keys);
    if !cfg.mcp_token.is_empty() {
        allowed_api_keys.push(cfg.mcp_token.clone());
    }

    let graph = load_graph()?;
    let mut mcp = build_mcp(graph);
    mcp.try_attach_rag();

    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .with_context(|| format!("invalid bind address: {host}:{port}"))?;
    let app = episteme::server::mcp_transport_http::mcp_http_router(mcp, allowed_api_keys);

    println!("episteme MCP server (HTTP): http://{host}:{port}/mcp");
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
    let handler = std::sync::Arc::new(build_mcp(graph));
    let app = episteme::server::web_viewer::web_router(handler);

    let addr = format!("{host}:{port}");

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        println!("Graph viewer at http://{addr}");
        axum::serve(listener, app).await?;
        Ok(())
    })
}

/// Build an [`EpistemeMCP`] handler from a knowledge graph.
///
/// Attempts to open the user knowledge database for composite graph support.
/// Falls back to a basic handler if the database is unavailable.
pub fn build_mcp(graph: episteme::domain::graph::KnowledgeGraph) -> EpistemeMCP {
    let db_path = paths::episteme_home().join("user_knowledge.db");
    match UserGraphStore::open(&db_path) {
        Ok(store) => {
            let composite = CompositeGraph::new(graph.clone(), Box::new(store));
            EpistemeMCP::with_composite(graph, composite)
        }
        Err(_) => EpistemeMCP::new(graph),
    }
}
