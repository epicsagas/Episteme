use crate::domain::metrics::SmellDetection;
use std::fs;
use std::path::Path;

/// Trait for language-specific code parsers.
///
/// Ported from `syntagma.parsers.base.LanguageParser`.
pub trait CodeParser: Send + Sync {
    /// Parse source code and return detected smells.
    fn parse_code(&self, code: &str, file_name: &str) -> Vec<SmellDetection>;

    /// Parse a file from disk and return detected smells.
    /// Returns an error if the file cannot be read.
    fn parse_file(&self, path: &Path) -> Result<Vec<SmellDetection>, std::io::Error> {
        let code = fs::read_to_string(path)?;
        let file_name = path.to_string_lossy().to_string();
        Ok(self.parse_code(&code, &file_name))
    }

    /// File extensions this parser handles (e.g., `["py"]`, `["java"]`).
    fn supported_extensions(&self) -> &[&str];
}
