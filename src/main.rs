use std::time::Instant;

use anyhow::Result;
use clap::Parser;

mod cli;
mod commands;

use cli::{Commands, GraphCommands, HooksCommands, ServiceCommands};

// ---------------------------------------------------------------------------
// CLI top-level struct
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(
    name = "epis",
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
        } => commands::cmd_explore(query, limit, entity_type.as_deref(), interactive),

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

        Commands::Api { host, port } => commands::cmd_api(&host, port),

        Commands::Service { sub } => commands::cmd_service(service_op(sub)),

        Commands::Mcp { http, host, port } => commands::cmd_mcp(http, &host, port),

        Commands::Telemetry { action } => commands::cmd_telemetry(&action),

        Commands::Stats => commands::cmd_stats(),

        Commands::Hooks { sub } => commands::cmd_hooks(hooks_op(sub)),

        Commands::Web { host, port } => commands::cmd_web(&host, port),

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

fn service_op(sub: ServiceCommands) -> commands::ServiceOp {
    match sub {
        ServiceCommands::Serve { host, port } => commands::ServiceOp::Serve { host, port },
        ServiceCommands::Start { host, port } => commands::ServiceOp::Start { host, port },
        ServiceCommands::Stop => commands::ServiceOp::Stop,
        ServiceCommands::Restart { host, port } => commands::ServiceOp::Restart { host, port },
        ServiceCommands::Status => commands::ServiceOp::Status,
        ServiceCommands::LaunchdInstall { host, port } => {
            commands::ServiceOp::LaunchdInstall { host, port }
        }
        ServiceCommands::LaunchdUninstall => commands::ServiceOp::LaunchdUninstall,
        ServiceCommands::LaunchdStatus => commands::ServiceOp::LaunchdStatus,
        ServiceCommands::Enable { now } => commands::ServiceOp::Enable { now },
        ServiceCommands::Disable { now } => commands::ServiceOp::Disable { now },
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
