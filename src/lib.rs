pub mod adapters;
pub mod domain;
pub mod ports;
pub mod server;

// Re-export primary types for convenience
pub use adapters::config::EpistemeConfig;
pub use adapters::paths;
pub use domain::graph::KnowledgeGraph;
pub use domain::metrics::{CodeMetrics, SmellDetection};
pub use domain::types::*;
pub use server::mcp_handler::EpistemeMCP;
