// Root-level modules
pub mod builder;
pub mod error;
pub mod hooks;
pub mod insight_utils;
pub mod json_loader;
pub mod metrics;
pub mod structured_logging;
pub mod telemetry;

// Sub-module groups
pub mod embeddings;
pub mod infra;
pub mod parsing;
pub mod setup;
pub mod web;

// Backward-compat re-exports: embeddings
pub use embeddings::chunker;
pub use embeddings::embedding_providers;
pub use embeddings::noop_embeddings;

// Backward-compat re-exports: parsing
pub use parsing::python_ast_parser;
pub use parsing::regex_parsers;

// Backward-compat re-exports: infra
pub use infra::cache;
pub use infra::search_engines;
pub use infra::service;
pub use infra::sqlite_db;
pub use infra::user_graph_store;

// Backward-compat re-exports: setup
pub use setup::config;
pub use setup::constants;
pub use setup::install_wizard;
pub use setup::installer;
pub use setup::paths;

// Backward-compat re-exports: web
pub use web::rate_limiter;
pub use web::rate_limiter_mw;

// Public type re-exports
pub use config::EpistemeConfig;
pub use embedding_providers::LkEmbeddingAdapter;
pub use error::InfraError;
pub use json_loader::load_graph;
pub use noop_embeddings::NoopEmbeddingProvider;
pub use regex_parsers::get_parser;
pub use user_graph_store::UserGraphStore;
