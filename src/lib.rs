pub mod domain;
pub mod ports;
pub mod adapters;
pub mod server;

// Re-export primary types for convenience
pub use domain::types::*;
pub use domain::graph::KnowledgeGraph;
pub use domain::metrics::{CodeMetrics, SmellDetection};
pub use adapters::config::SyntagmaConfig;
pub use adapters::paths;
pub use server::mcp_handler::SyntagmaMCP;
