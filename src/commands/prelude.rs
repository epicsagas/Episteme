//! Shared helpers used by multiple command modules.

use std::path::PathBuf;

use anyhow::{Context, Result};

use syntagma::adapters::paths;
use syntagma::domain::graph::KnowledgeGraph;

/// Load the knowledge graph from the default data directory.
pub fn load_graph() -> Result<KnowledgeGraph> {
    let dir = paths::data_dir();
    syntagma::adapters::json_loader::load_graph(&dir)
        .with_context(|| format!("failed to load knowledge graph from {}", dir.display()))
}

/// Resolve the default data directory (or use override).
pub fn resolve_data_dir(override_dir: Option<&str>) -> PathBuf {
    match override_dir {
        Some(d) => PathBuf::from(d),
        None => paths::data_dir(),
    }
}

/// Detect language from file extension when not explicitly provided.
pub fn detect_language(file: &str, language: Option<&str>) -> Result<String> {
    if let Some(lang) = language {
        return Ok(lang.to_owned());
    }
    let ext = std::path::Path::new(file)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext {
        "py" => Ok("python".to_owned()),
        "java" => Ok("java".to_owned()),
        "go" => Ok("go".to_owned()),
        "rs" => Ok("rust".to_owned()),
        "ts" | "tsx" => Ok("typescript".to_owned()),
        "js" | "jsx" => Ok("javascript".to_owned()),
        "c" | "cpp" | "cxx" | "cc" | "hpp" | "h" => Ok("cpp".to_owned()),
        "cs" => Ok("csharp".to_owned()),
        "kt" | "kts" => Ok("kotlin".to_owned()),
        "php" => Ok("php".to_owned()),
        "rb" => Ok("ruby".to_owned()),
        other => anyhow::bail!(
            "cannot detect language from extension '.{}'. Please specify --language.",
            other
        ),
    }
}

/// Try to extract a u64 from a JSON value, handling both Number and String.
pub fn as_u64_or_zero(v: &serde_json::Value) -> Option<u64> {
    v.as_u64()
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
}
