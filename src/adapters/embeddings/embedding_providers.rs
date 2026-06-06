//! Thin adapter wrapping [`llm_kernel::embedding::EmbeddingProvider`] behind
//! Episteme's [`crate::ports::embeddings::EmbeddingProvider`] trait.
//!
//! This module replaces the previous direct fastembed + reqwest implementations
//! with llm-kernel's unified embedding providers.

use llm_kernel::embedding::EmbeddingProvider as LkProvider;

use crate::ports::embeddings::EmbeddingProvider;

/// Adapter wrapping an [`LkProvider`] behind Episteme's [`EmbeddingProvider`] trait.
///
/// Translates between the two trait signatures:
/// - `dim()` → `embedding_dim()`
/// - `anyhow::Result<EmbeddingResult>` → `Result<Vec<f32>, String>`
/// - `embed_batch(&[&str])` → `embed_batch(&[&str], batch_size)` with manual chunking
pub struct LkEmbeddingAdapter {
    inner: Box<dyn LkProvider>,
}

impl LkEmbeddingAdapter {
    pub fn new(inner: Box<dyn LkProvider>) -> Self {
        Self { inner }
    }
}

impl EmbeddingProvider for LkEmbeddingAdapter {
    fn embedding_dim(&self) -> usize {
        self.inner.dim()
    }

    fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
        self.inner
            .embed(text)
            .map(|r| r.vector)
            .map_err(|e| e.to_string())
    }

    fn embed_batch(&self, texts: &[&str], batch_size: usize) -> Result<Vec<Vec<f32>>, String> {
        if batch_size == 0 || batch_size >= texts.len() {
            return self
                .inner
                .embed_batch(texts)
                .map(|results| results.into_iter().map(|r| r.vector).collect())
                .map_err(|e| e.to_string());
        }

        // Chunk manually to respect batch_size memory limits.
        let mut all = Vec::with_capacity(texts.len());
        for chunk in texts.chunks(batch_size) {
            let batch = self.inner.embed_batch(chunk).map_err(|e| e.to_string())?;
            all.extend(batch.into_iter().map(|r| r.vector));
        }
        Ok(all)
    }
}

/// Create a local fastembed-based embedding provider using llm-kernel.
///
/// `model_name` is parsed case-insensitively via [`llm_kernel::embedding::catalog::EmbeddingModel::parse`].
/// Falls back to `MultilingualE5Small` (384-dim) on parse failure.
pub fn create_local_provider(model_name: &str) -> Result<Box<dyn EmbeddingProvider>, String> {
    use llm_kernel::embedding::FastembedProvider;
    use llm_kernel::embedding::catalog::EmbeddingModel;

    let model = EmbeddingModel::parse(model_name).unwrap_or(EmbeddingModel::MultilingualE5Small);
    let cache_dir = crate::adapters::paths::episteme_home().join("models");

    let provider = FastembedProvider::new(model, Some(cache_dir))
        .map_err(|e| format!("failed to create embedding provider: {e}"))?;

    Ok(Box::new(LkEmbeddingAdapter::new(Box::new(provider))))
}

/// Create a local provider reading the model from config (`EPISTEME_EMBEDDING_MODEL`).
///
/// Falls back to `NoopEmbeddingProvider` when model loading fails, using the configured
/// model's dimension so stored vectors remain compatible with the noop fallback.
pub fn create_configured_local_provider() -> Box<dyn EmbeddingProvider> {
    use llm_kernel::embedding::catalog::EmbeddingModel;

    let cfg = crate::adapters::config::EpistemeConfig::load().unwrap_or_default();
    let fallback_dim = EmbeddingModel::parse(&cfg.embedding_model)
        .unwrap_or(EmbeddingModel::MultilingualE5Small)
        .dimension();
    match create_local_provider(&cfg.embedding_model) {
        Ok(provider) => provider,
        Err(e) => {
            tracing::warn!(
                "Failed to load local embedding model '{}', falling back to noop provider ({fallback_dim}-dim): {e}",
                cfg.embedding_model
            );
            Box::new(crate::adapters::noop_embeddings::NoopEmbeddingProvider::new(fallback_dim))
        }
    }
}

/// Create an OpenAI embedding provider using llm-kernel.
///
/// `model` is matched against known OpenAI model names to pick the right constructor.
/// Falls back to `new_small` for unrecognized model names.
#[cfg(feature = "openai-embeddings")]
pub fn create_openai_provider(
    api_key: String,
    model: String,
) -> Result<Box<dyn EmbeddingProvider>, String> {
    use llm_kernel::embedding::OpenAIEmbeddingClient;

    let provider = if model.contains("large") {
        OpenAIEmbeddingClient::new_large(api_key)
    } else {
        OpenAIEmbeddingClient::new_small(api_key)
    };

    Ok(Box::new(LkEmbeddingAdapter::new(Box::new(provider))))
}

#[cfg(test)]
mod tests {
    use llm_kernel::embedding::types::{EmbeddingProvider as LkProvider, EmbeddingResult};

    use super::*;

    /// Mock that implements llm-kernel's `EmbeddingProvider` for unit testing.
    struct MockLkProvider {
        dim: usize,
    }

    impl LkProvider for MockLkProvider {
        fn dim(&self) -> usize {
            self.dim
        }

        fn name(&self) -> &str {
            "mock"
        }

        fn embed(&self, text: &str) -> anyhow::Result<EmbeddingResult> {
            Ok(EmbeddingResult {
                vector: vec![1.0f32; self.dim],
                text_preview: text[..text.len().min(64)].to_string(),
            })
        }

        fn embed_batch(&self, texts: &[&str]) -> anyhow::Result<Vec<EmbeddingResult>> {
            Ok(texts
                .iter()
                .map(|t| EmbeddingResult {
                    vector: vec![1.0f32; self.dim],
                    text_preview: t[..t.len().min(64)].to_string(),
                })
                .collect())
        }
    }

    fn mock_adapter(dim: usize) -> LkEmbeddingAdapter {
        LkEmbeddingAdapter::new(Box::new(MockLkProvider { dim }))
    }

    #[test]
    fn adapter_dim_delegates() {
        let adapter = mock_adapter(384);
        assert_eq!(adapter.embedding_dim(), 384);
    }

    #[test]
    fn adapter_embed_delegates() {
        let adapter = mock_adapter(128);
        let vec = adapter.embed("test").unwrap();
        assert_eq!(vec.len(), 128);
        assert!(vec.iter().all(|&v| v == 1.0));
    }

    #[test]
    fn adapter_embed_batch_no_chunking() {
        let adapter = mock_adapter(64);
        // batch_size=0 → single call
        let results = adapter.embed_batch(&["a", "b"], 0).unwrap();
        assert_eq!(results.len(), 2);
        for v in &results {
            assert_eq!(v.len(), 64);
        }
    }

    #[test]
    fn adapter_embed_batch_with_chunking() {
        let adapter = mock_adapter(64);
        // 3 items, batch_size=2 → 2 chunks [a,b] + [c]
        let results = adapter.embed_batch(&["a", "b", "c"], 2).unwrap();
        assert_eq!(results.len(), 3);
    }
}
