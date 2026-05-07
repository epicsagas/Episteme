//! Command implementations, grouped by domain.
//!
//! Each sub-module handles one family of CLI subcommands.
//! The `prelude` module provides shared helpers (`load_graph`, `detect_language`, etc.).

pub mod prelude;

pub mod analysis;
pub mod build;
pub mod explore;
pub mod graph;
pub mod install;
pub mod other;
pub mod service;

// Re-export the primary entry-points so callers can `use crate::commands::cmd_foo`.

pub use analysis::{cmd_analyze, cmd_infer};
pub use build::{cmd_build, cmd_dist};
pub use explore::cmd_explore;
pub use graph::{GraphOp, cmd_graph};
pub use install::cmd_install;
pub use other::{HooksOp, cmd_hooks, cmd_stats, cmd_telemetry};
pub use service::{ServiceOp, cmd_api, cmd_mcp, cmd_service, cmd_web};
