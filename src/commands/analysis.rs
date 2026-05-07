//! Analysis commands: analyze and infer.

use std::path::PathBuf;

use anyhow::{Context, Result};

use syntagma::adapters::regex_parsers::get_parser;
use syntagma::domain::engine::RefactoringInferenceEngine;

use super::prelude::*;

pub fn cmd_analyze(
    file: &str,
    language: Option<&str>,
    json: bool,
    min_confidence: f64,
) -> Result<()> {
    let lang = detect_language(file, language)?;

    let parser = get_parser(&lang).map_err(|e| anyhow::anyhow!("{e}"))?;

    let path = PathBuf::from(file);
    let detections = parser
        .parse_file(&path)
        .with_context(|| format!("failed to parse {file}"))?;

    // Filter by minimum confidence when threshold > 0.
    let detections: Vec<_> = if min_confidence > 0.0 {
        detections
            .into_iter()
            .filter(|d| d.confidence >= min_confidence)
            .collect()
    } else {
        detections
    };

    if detections.is_empty() {
        if json {
            println!("{}", serde_json::json!({"smells": [], "count": 0}));
        } else {
            println!("No code smells detected in {file}.");
        }
        return Ok(());
    }

    if json {
        let json_val = serde_json::json!({
            "file": file,
            "language": lang,
            "smells": detections,
            "count": detections.len(),
        });
        println!("{}", serde_json::to_string_pretty(&json_val)?);
    } else {
        println!("Detected {} code smell(s) in {}:", detections.len(), file);
        println!();
        for d in &detections {
            println!(
                "  [{}] {} ({:.0}% confidence)",
                d.smell_id,
                d.smell_name,
                d.confidence * 100.0
            );
            println!("    Location: {}", d.location);
            println!("    Function: {}", d.function_name);
            if !d.reasons.is_empty() {
                println!("    Reasons:  {}", d.reasons.join("; "));
            }
            println!();
        }
    }

    Ok(())
}

pub fn cmd_infer(file: &str, language: Option<&str>, top_k: usize, json: bool) -> Result<()> {
    let lang = detect_language(file, language)?;

    let parser = get_parser(&lang).map_err(|e| anyhow::anyhow!("{e}"))?;

    let path = PathBuf::from(file);
    let detections = parser
        .parse_file(&path)
        .with_context(|| format!("failed to parse {file}"))?;

    if detections.is_empty() {
        if json {
            println!("{}", serde_json::json!({"analyses": [], "count": 0}));
        } else {
            println!("No code smells detected in {file}.");
        }
        return Ok(());
    }

    if !json {
        println!(
            "Detected {} code smell(s). Loading knowledge graph for refactoring suggestions...",
            detections.len()
        );
    }

    let graph = load_graph()?;
    let engine = RefactoringInferenceEngine::new(graph);
    let analyses = engine.analyze_detections(&detections, top_k);

    if json {
        let json_val = serde_json::json!({
            "file": file,
            "language": lang,
            "analyses": analyses,
            "count": analyses.len(),
        });
        println!("{}", serde_json::to_string_pretty(&json_val)?);
        return Ok(());
    }

    for analysis in &analyses {
        let smell = &analysis.smell;
        let smell_name = smell
            .get("smell_name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        let smell_id = smell
            .get("smell_id")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let location = smell
            .get("location")
            .and_then(|v| v.as_str())
            .unwrap_or("?");

        println!("[{}] {} at {}", smell_id, smell_name, location);

        if analysis.suggestions.is_empty() {
            println!("  No refactoring suggestions found in the knowledge graph.");
        } else {
            for (i, s) in analysis.suggestions.iter().enumerate() {
                println!(
                    "  {}. {} ({}) - priority: {:.3}, effort: {}",
                    i + 1,
                    s.title,
                    s.refactoring_id,
                    s.priority_score,
                    s.effort
                );
                if !s.principles_enforced.is_empty() {
                    println!("     Enforces: {}", s.principles_enforced.join(", "));
                }
                println!("     {}", s.description);
            }
        }
        println!();
    }

    Ok(())
}
