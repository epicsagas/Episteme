//! Build and dist commands.

use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use tracing::info;

use syntagma::adapters::config::SyntagmaConfig;
use syntagma::adapters::constants::EMBEDDING_DIMENSIONS;

use super::prelude::*;

pub fn cmd_build(
    data_dir: Option<&str>,
    gpu: bool,
    no_gpu: bool,
    batch_size: usize,
    rebuild: bool,
    print_stats: bool,
) -> Result<()> {
    let config = SyntagmaConfig::load()?;

    let data_dir = resolve_data_dir(data_dir);
    let raw_dir = syntagma::adapters::paths::raw_dir();
    let db_path = syntagma::adapters::paths::db_path();

    // If --rebuild, delete the existing database.
    if rebuild && db_path.exists() {
        info!("Removing existing database at {}", db_path.display());
        std::fs::remove_file(&db_path)
            .with_context(|| format!("failed to remove {}", db_path.display()))?;
    }

    info!(
        "Building RAG index from {} (raw: {}, db: {})",
        data_dir.display(),
        raw_dir.display(),
        db_path.display()
    );

    // Ensure directories exist.
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating db directory {}", parent.display()))?;
    }

    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_style(
        indicatif::ProgressStyle::default_spinner()
            .template("{spinner} [{elapsed}] {msg}")
            .unwrap(),
    );
    pb.set_message("Building RAG index...");

    let provider: Box<dyn syntagma::ports::embeddings::EmbeddingProvider> = {
        #[cfg(feature = "openai-embeddings")]
        {
            let provider_pref = config.embedding_provider.to_lowercase();
            let key = std::env::var("SYNTAGMA_OPENAI_API_KEY")
                .ok()
                .filter(|k| !k.is_empty())
                .or_else(|| {
                    let k = config.openai_api_key.clone();
                    if k.is_empty() { None } else { Some(k) }
                });

            if provider_pref == "openai" {
                if let Some(key) = key {
                    let model = std::env::var("SYNTAGMA_OPENAI_EMBED_MODEL")
                        .ok()
                        .filter(|m| !m.is_empty())
                        .unwrap_or(config.openai_embed_model.clone());
                    let dim = std::env::var("SYNTAGMA_OPENAI_EMBED_DIM")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(config.openai_embed_dim);
                    info!("Using OpenAI embedding provider (model={model}, dim={dim})");
                    Box::new(
                        syntagma::adapters::openai_embeddings::OpenAIEmbeddingProvider::new(
                            key, model, dim,
                        ),
                    )
                } else {
                    anyhow::bail!(
                        "embedding provider is set to openai but no API key was found (set OPENAI_API_KEY or SYNTAGMA_OPENAI_API_KEY)"
                    );
                }
            } else {
                info!("Using local embedding provider");
                Box::new(syntagma::adapters::local_embeddings::LocalEmbeddingProvider::new(
                    EMBEDDING_DIMENSIONS,
                ))
            }
        }

        #[cfg(not(feature = "openai-embeddings"))]
        {
            let _ = &config;
            Box::new(syntagma::adapters::local_embeddings::LocalEmbeddingProvider::new(
                EMBEDDING_DIMENSIONS,
            ))
        }
    };

    if gpu && no_gpu {
        anyhow::bail!("--gpu and --no-gpu cannot be used together");
    }
    if gpu {
        info!("--gpu requested; provider selection is controlled by build features and env");
    } else if no_gpu {
        info!("--no-gpu requested; CPU-only behavior applies for local embedding providers");
    }

    let stats = syntagma::adapters::builder::build(
        &db_path,
        &data_dir,
        &raw_dir,
        provider.as_ref(),
        batch_size,
    )
    .with_context(|| "RAG build failed")?;

    pb.finish_with_message("done");

    println!("RAG build complete:");
    println!("  Files scanned:          {}", stats.files_scanned);
    println!("  Chunks created:         {}", stats.chunks_created);
    println!("  Embeddings generated:   {}", stats.embeddings_generated);
    println!("  Skipped (no file):      {}", stats.skipped_no_file);

    // If --stats, print graph statistics after build.
    if print_stats {
        println!();
        match load_graph() {
            Ok(graph) => {
                let gs = graph.stats();
                println!("Knowledge Graph Statistics:");
                println!("  Total entities:        {}", gs.total_entities);
                println!("  Total edges:           {}", gs.total_edges);
                println!("  With relations:        {}", gs.entities_with_relations);
                println!(
                    "  Avg edges/entity:      {:.2}",
                    gs.avg_edges_per_entity
                );
            }
            Err(e) => {
                println!("Could not load graph for stats: {e:#}");
            }
        }
    }

    Ok(())
}

pub fn cmd_dist(out_dir: &str, no_db: bool, skip_build: bool) -> Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    let package_name = format!("syntagma-data-{version}");
    let out_dir = PathBuf::from(out_dir);
    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create output dir {}", out_dir.display()))?;

    let staging_root = std::env::temp_dir().join(format!(
        "syntagma-dist-{}-{}",
        std::process::id(),
        version
    ));
    if staging_root.exists() {
        let _ = std::fs::remove_dir_all(&staging_root);
    }
    let package_root = staging_root.join(&package_name);
    std::fs::create_dir_all(&package_root)?;

    let cwd = std::env::current_dir().context("failed to resolve current directory")?;
    let mut copied_any = false;

    for dir in ["raw", "meta", "data"] {
        let src = cwd.join(dir);
        if src.exists() && src.is_dir() {
            let dst = package_root.join(dir);
            copy_dir_recursive(&src, &dst)?;
            copied_any = true;
        }
    }
    if !copied_any {
        anyhow::bail!("no raw/meta/data directories found in {}", cwd.display());
    }

    if !no_db {
        let db_src = syntagma::adapters::paths::db_path();
        if !db_src.exists() && !skip_build {
            println!(
                "Embedding DB not found at {}. Running `syntagma build` first...",
                db_src.display()
            );
            cmd_build(None, false, false, 64, false, false)?;
        }
        if !db_src.exists() {
            anyhow::bail!(
                "embedding db not found at {} (auto-build skipped/failed; rerun without --skip-build or use --no-db)",
                db_src.display()
            );
        }
        let db_dst_dir = package_root.join("db");
        std::fs::create_dir_all(&db_dst_dir)?;
        std::fs::copy(&db_src, db_dst_dir.join("syntagma.db"))
            .with_context(|| format!("failed to copy db from {}", db_src.display()))?;

        // Compatibility with older package expectations.
        let meta_dir = package_root.join("meta");
        std::fs::create_dir_all(&meta_dir)?;
        std::fs::copy(&db_src, meta_dir.join("syntagma.db"))
            .with_context(|| format!("failed to copy db into {}", meta_dir.display()))?;
    }

    let archive_path = out_dir.join(format!("{package_name}.tar.gz"));
    let tar_gz = std::fs::File::create(&archive_path)
        .with_context(|| format!("failed to create {}", archive_path.display()))?;
    let enc = flate2::write::GzEncoder::new(tar_gz, flate2::Compression::default());
    let mut tar = tar::Builder::new(enc);
    tar.append_dir_all(&package_name, &package_root)
        .with_context(|| format!("failed to archive {}", package_root.display()))?;
    let enc = tar.into_inner().context("failed to finalize tar stream")?;
    let mut file = enc.finish().context("failed to finalize gzip stream")?;
    file.flush().ok();

    let _ = std::fs::remove_dir_all(&staging_root);

    println!("Created dist archive: {}", archive_path.display());
    println!("Included: raw/meta/data{}", if no_db { "" } else { ", db/syntagma.db" });
    Ok(())
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    std::fs::create_dir_all(dst)
        .with_context(|| format!("failed to create {}", dst.display()))?;
    for entry in std::fs::read_dir(src)
        .with_context(|| format!("failed to read {}", src.display()))?
    {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path).with_context(|| {
                format!("failed to copy {} -> {}", src_path.display(), dst_path.display())
            })?;
        }
    }
    Ok(())
}
