//! CLI hooks for AI-assisted coding workflows.
//!
//! Ported from Python `cli/hooks.py`.
//!
//! Each hook emits XML-comment blocks that downstream tools (editors, CI)
//! can parse without affecting the surrounding text.

use std::path::Path;

use crate::adapters::regex_parsers::get_parser;
use crate::domain::graph::KnowledgeGraph;

// ---------------------------------------------------------------------------
// Language detection (mirrors the one in main.rs)
// ---------------------------------------------------------------------------

/// Map a file extension to a language identifier recognized by the parser
/// registry. Returns an empty string for unknown extensions.
pub fn detect_language_from_ext(ext: &str) -> &'static str {
    match ext {
        "py" => "python",
        "java" => "java",
        "go" => "go",
        "rs" => "rust",
        "ts" | "tsx" => "typescript",
        "js" | "jsx" => "javascript",
        "c" | "cpp" | "cxx" | "cc" | "hpp" | "h" => "cpp",
        "cs" => "csharp",
        "kt" | "kts" => "kotlin",
        "php" => "php",
        "rb" => "ruby",
        _ => "",
    }
}

// ---------------------------------------------------------------------------
// Ground hook
// ---------------------------------------------------------------------------

/// Search the knowledge graph for entities relevant to `prompt`.
///
/// Returns an XML-comment block that downstream consumers can embed.
pub fn handle_ground(graph: &KnowledgeGraph, prompt: &str, limit: usize) -> String {
    let prompt_lower = prompt.to_lowercase();
    let prompt_words: Vec<&str> = prompt_lower.split_whitespace().collect();

    let mut results: Vec<(String, usize)> = Vec::new();

    for id in graph.all_entity_ids() {
        let Some(entity) = graph.get_entity(&id) else {
            continue;
        };

        let text = format!(
            "{} {} {} {}",
            entity.title,
            entity.name,
            entity.category,
            entity.tags.join(" ")
        );
        let text_lower = text.to_lowercase();

        let matches = prompt_words
            .iter()
            .filter(|w| text_lower.contains(*w))
            .count();

        if matches > 0 {
            results.push((id, matches));
        }
    }

    results.sort_by_key(|b| std::cmp::Reverse(b.1));
    results.truncate(limit);

    let mut output = String::from("<!-- episteme-ground -->\n");
    for (id, _score) in &results {
        if let Some(entity) = graph.get_entity(id) {
            output.push_str(&format!(
                "<!-- {} | {} | {} -->\n",
                id, entity.title, entity.category
            ));
        }
    }
    output.push_str("<!-- /episteme-ground -->\n");
    output
}

// ---------------------------------------------------------------------------
// Sniff hook
// ---------------------------------------------------------------------------

/// Detect code smells in the given files.
///
/// Returns an XML-comment block listing every detection above
/// `min_confidence`.
pub fn handle_sniff(files: &[String], min_confidence: f64) -> String {
    let mut output = String::from("<!-- episteme-sniff -->\n");

    for file in files {
        let path = Path::new(file);
        if !path.exists() {
            continue;
        }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let lang = detect_language_from_ext(ext);
        if lang.is_empty() {
            continue;
        }

        let Ok(parser) = get_parser(lang) else {
            continue;
        };

        let detections = match parser.parse_file(path) {
            Ok(d) => d,
            Err(_) => continue,
        };

        for d in &detections {
            if d.confidence >= min_confidence {
                output.push_str(&format!(
                    "<!-- SMELL: {} ({}) in {} at {} confidence={:.2} -->\n",
                    d.smell_name, d.smell_id, file, d.function_name, d.confidence
                ));
            }
        }
    }

    output.push_str("<!-- /episteme-sniff -->\n");
    output
}

// ---------------------------------------------------------------------------
// Audit hook
// ---------------------------------------------------------------------------

/// Final quality audit: analyse a single file and emit a pass/fail block.
pub fn handle_audit(file_path: Option<&str>, min_confidence: f64) -> String {
    let detections = if let Some(fp) = file_path {
        let path = Path::new(fp);
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let lang = detect_language_from_ext(ext);
        if lang.is_empty() {
            return "<!-- episteme-audit: no language detected -->\n".to_owned();
        }
        get_parser(lang)
            .ok()
            .and_then(|p| p.parse_file(path).ok())
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let mut output = String::from("<!-- episteme-audit -->\n");

    let high: Vec<_> = detections
        .iter()
        .filter(|d| d.confidence >= min_confidence && d.confidence >= 0.7)
        .collect();

    let medium: Vec<_> = detections
        .iter()
        .filter(|d| d.confidence >= min_confidence && d.confidence >= 0.5 && d.confidence < 0.7)
        .collect();

    if high.is_empty() && medium.is_empty() {
        output.push_str("<!-- PASS: No significant code smells detected -->\n");
    } else {
        for d in high {
            output.push_str(&format!(
                "<!-- WARN: {} ({}) confidence={:.2} -->\n",
                d.smell_name, d.smell_id, d.confidence
            ));
        }
        for d in medium {
            output.push_str(&format!(
                "<!-- INFO: {} ({}) confidence={:.2} -->\n",
                d.smell_name, d.smell_id, d.confidence
            ));
        }
    }

    output.push_str("<!-- /episteme-audit -->\n");
    output
}

// ---------------------------------------------------------------------------
// Git helpers
// ---------------------------------------------------------------------------

/// Return the list of files currently staged in the git index.
pub fn get_staged_files() -> Vec<String> {
    std::process::Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .output()
        .ok()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter(|l| !l.is_empty())
                .map(|l| l.to_owned())
                .collect()
        })
        .unwrap_or_default()
}
