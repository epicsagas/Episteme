use std::time::Instant;

use anyhow::Result;
use clap::Parser;

mod cli;
mod commands;

use episteme::adapters::service::ServiceKind;

use cli::{
    Commands, GraphCommands, HooksCommands, InsightCommands, ServiceCommands, ServiceLifecycle,
};

// ---------------------------------------------------------------------------
// CLI top-level struct
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(
    name = "episteme",
    version,
    about = "Software engineering knowledge graph"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() -> Result<()> {
    let cli = Cli::parse();

    install_tracing();
    let _ = episteme::adapters::telemetry::ensure_consent_or_set_default();
    episteme::adapters::telemetry::track_session_started();

    let cmd_enum = telemetry_command(&cli.command);
    if let Some(cmd) = cmd_enum {
        episteme::adapters::telemetry::track_command_invoked(cmd);
    }
    let started_at = Instant::now();

    let result = dispatch(cli);

    if let Some(cmd) = cmd_enum {
        let elapsed = started_at.elapsed().as_millis();
        match &result {
            Ok(_) => episteme::adapters::telemetry::track_command_completed(cmd, elapsed),
            Err(_) => episteme::adapters::telemetry::track_command_failed(
                cmd,
                episteme::adapters::telemetry::FailureClass::Unknown,
            ),
        }
    }

    result
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

fn dispatch(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Analyze {
            file,
            language,
            json,
            min_confidence,
        } => commands::cmd_analyze(&file, language.as_deref(), json, min_confidence),

        Commands::Infer {
            file,
            language,
            top_k,
            json,
        } => commands::cmd_infer(&file, language.as_deref(), top_k, json),

        Commands::Explore {
            query,
            limit,
            entity_type,
            interactive,
        } => {
            warn_deprecated("explore", "search");
            commands::cmd_explore(query, limit, entity_type.as_deref(), interactive)
        }

        Commands::Graph { sub } => commands::cmd_graph(graph_op(sub)),

        Commands::Build {
            data_dir,
            gpu,
            no_gpu,
            batch_size,
            rebuild,
            stats,
        } => commands::cmd_build(
            data_dir.as_deref(),
            None,
            gpu,
            no_gpu,
            batch_size,
            rebuild,
            stats,
        ),

        Commands::Dist {
            out_dir,
            no_db,
            skip_build,
        } => commands::cmd_dist(&out_dir, no_db, skip_build),

        Commands::Api { sub, host, port } => match sub {
            Some(api_sub) => commands::cmd_service(service_lifecycle_op(api_sub, ServiceKind::Api)),
            None => commands::cmd_api(&host, port),
        },

        Commands::Service { sub } => {
            eprintln!(
                "[deprecated] 'service' is deprecated, use 'mcp start/stop/restart/status/enable/disable' instead."
            );
            commands::cmd_service(legacy_service_op(sub))
        }

        Commands::Mcp {
            sub,
            http,
            host,
            port,
        } => match sub {
            Some(mcp_sub) => commands::cmd_service(service_lifecycle_op(mcp_sub, ServiceKind::Mcp)),
            None => commands::cmd_mcp(http, &host, port),
        },

        Commands::Telemetry { action } => commands::cmd_telemetry(&action),

        Commands::Stats => commands::cmd_stats(),

        Commands::Hooks { sub } => commands::cmd_hooks(hooks_op(sub)),

        Commands::Web { host, port } => commands::cmd_web(&host, port),

        Commands::Insight { sub } => commands::cmd_insight(insight_op(sub)),

        Commands::Install {
            tools,
            all,
            dry_run,
            local,
        } => commands::cmd_install(&tools, all, dry_run, local),
    }
}

// ---------------------------------------------------------------------------
// Enum converters: clap enums -> command-internal dispatch types
// ---------------------------------------------------------------------------

fn graph_op(sub: GraphCommands) -> commands::GraphOp {
    match sub {
        GraphCommands::Entity { id } => commands::GraphOp::Entity { id },
        GraphCommands::Neighbors { id, relation_type } => {
            commands::GraphOp::Neighbors { id, relation_type }
        }
        GraphCommands::Path {
            from,
            to,
            max_depth,
        } => commands::GraphOp::Path {
            from,
            to,
            max_depth,
        },
        GraphCommands::Contradictions => commands::GraphOp::Contradictions,
    }
}

fn legacy_service_op(sub: ServiceCommands) -> commands::ServiceOp {
    // Legacy 'service' commands always target MCP.
    match sub {
        ServiceCommands::Serve { host, port } => commands::ServiceOp::Serve {
            host,
            port,
            kind: ServiceKind::Mcp,
        },
        ServiceCommands::Start { host, port } => commands::ServiceOp::Start {
            host,
            port,
            kind: ServiceKind::Mcp,
        },
        ServiceCommands::Stop => commands::ServiceOp::Stop {
            kind: ServiceKind::Mcp,
        },
        ServiceCommands::Restart { host, port } => commands::ServiceOp::Restart {
            host,
            port,
            kind: ServiceKind::Mcp,
        },
        ServiceCommands::Status => commands::ServiceOp::Status {
            kind: ServiceKind::Mcp,
        },
        ServiceCommands::LaunchdInstall { host, port } => {
            commands::ServiceOp::LaunchdInstall { host, port }
        }
        ServiceCommands::LaunchdUninstall => commands::ServiceOp::LaunchdUninstall,
        ServiceCommands::LaunchdStatus => commands::ServiceOp::LaunchdStatus,
        ServiceCommands::Enable { now } => commands::ServiceOp::Enable {
            now,
            kind: ServiceKind::Mcp,
        },
        ServiceCommands::Disable { now } => commands::ServiceOp::Disable {
            now,
            kind: ServiceKind::Mcp,
        },
    }
}

fn service_lifecycle_op(sub: ServiceLifecycle, kind: ServiceKind) -> commands::ServiceOp {
    match sub {
        ServiceLifecycle::Start { host, port } => {
            let (def_host, def_port) = match kind {
                ServiceKind::Mcp => ("127.0.0.1", 43175),
                ServiceKind::Api => ("0.0.0.0", 8000),
            };
            commands::ServiceOp::Start {
                host: host.unwrap_or_else(|| def_host.to_string()),
                port: port.unwrap_or(def_port),
                kind,
            }
        }
        ServiceLifecycle::Stop => commands::ServiceOp::Stop { kind },
        ServiceLifecycle::Restart { host, port } => {
            let (def_host, def_port) = match kind {
                ServiceKind::Mcp => ("127.0.0.1", 43175),
                ServiceKind::Api => ("0.0.0.0", 8000),
            };
            commands::ServiceOp::Restart {
                host: host.unwrap_or_else(|| def_host.to_string()),
                port: port.unwrap_or(def_port),
                kind,
            }
        }
        ServiceLifecycle::Status => commands::ServiceOp::Status { kind },
        ServiceLifecycle::Enable { now } => commands::ServiceOp::Enable { now, kind },
        ServiceLifecycle::Disable { now } => commands::ServiceOp::Disable { now, kind },
        ServiceLifecycle::Serve { host, port } => commands::ServiceOp::Serve { host, port, kind },
    }
}


fn hooks_op(sub: HooksCommands) -> commands::HooksOp {
    match sub {
        HooksCommands::Ground {
            prompt,
            limit,
            json,
        } => commands::HooksOp::Ground {
            prompt,
            limit,
            json,
        },
        HooksCommands::Sniff {
            files,
            staged,
            min_confidence,
            json,
            verbose,
        } => commands::HooksOp::Sniff {
            files,
            staged,
            min_confidence,
            _json: json,
            verbose,
        },
        HooksCommands::Audit { file, json } => commands::HooksOp::Audit { file, _json: json },
    }
}

fn insight_op(sub: InsightCommands) -> commands::InsightOp {
    match sub {
        InsightCommands::Add {
            title,
            content,
            tags,
            link,
        } => commands::InsightOp::Add {
            title,
            content,
            tags,
            link,
        },
        InsightCommands::List { limit } => commands::InsightOp::List { limit },
        InsightCommands::Search { query, limit } => commands::InsightOp::Search { query, limit },
    }
}

// ---------------------------------------------------------------------------
// Telemetry mapping
// ---------------------------------------------------------------------------

/// Map a `Commands` variant to the telemetry `Command` enum.
///
/// Adding a new command only requires touching this function and the
/// `dispatch` match; the telemetry crate's `Command` enum is separate.
fn telemetry_command(cmd: &Commands) -> Option<episteme::adapters::telemetry::Command> {
    use episteme::adapters::telemetry::Command as C;
    match cmd {
        Commands::Install { .. } => Some(C::Install),
        Commands::Build { .. } => Some(C::Build),
        Commands::Analyze { .. } => Some(C::Analyze),
        Commands::Infer { .. } => Some(C::Infer),
        Commands::Explore { .. } => Some(C::Explore),
        Commands::Api { .. } => Some(C::Api),
        Commands::Mcp { .. } => Some(C::Mcp),
        Commands::Service { .. } => Some(C::Service),
        Commands::Telemetry { .. } => Some(C::Telemetry),
        Commands::Insight { .. } => None, // insight commands are local-only, skip telemetry
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn install_tracing() {
    use tracing_subscriber::EnvFilter;
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .try_init();
}

fn warn_deprecated(old: &str, new: &str) {
    if std::env::args().nth(1).is_some_and(|arg| arg == old) {
        eprintln!(
            "[deprecated] '{}' is deprecated, use '{}' instead.",
            old, new
        );
    }
}
