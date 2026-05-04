use crate::ports::embeddings::EmbeddingProvider;
use fastembed::{EmbeddingModel, TextEmbedding, TextInitOptions};
use std::sync::Mutex;
use tracing::warn;

/// Lightweight local embedding provider.
///
/// This is a deterministic CPU-only fallback that avoids external APIs.
/// It hashes token n-grams into a fixed-size normalized vector, providing
/// better semantic separation than zero-vectors while remaining dependency-light.
pub struct LocalEmbeddingProvider {
    dim: usize,
    backend: LocalBackend,
}

enum LocalBackend {
    FastEmbed(Box<Mutex<TextEmbedding>>),
    HashOnly,
}

impl LocalEmbeddingProvider {
    pub fn new(dim: usize) -> Self {
        let dim = dim.max(8);
        let backend = match TextEmbedding::try_new(
            TextInitOptions::new(EmbeddingModel::AllMiniLML6V2).with_show_download_progress(false),
        ) {
            Ok(model) => LocalBackend::FastEmbed(Box::new(Mutex::new(model))),
            Err(err) => {
                warn!(
                    "Failed to initialize local embedding model, falling back to hash embeddings: {err}"
                );
                LocalBackend::HashOnly
            }
        };
        Self { dim, backend }
    }

    fn hash_embed(&self, text: &str) -> Vec<f32> {
        let mut v = vec![0.0f32; self.dim];
        if text.trim().is_empty() {
            return v;
        }

        let lower = text.to_lowercase();
        let words: Vec<&str> = lower.split_whitespace().collect();
        for (i, w) in words.iter().enumerate() {
            let h = fxhash::hash64(w.as_bytes()) as usize;
            v[h % self.dim] += 1.0;
            if i + 1 < words.len() {
                let bigram = format!("{w} {}", words[i + 1]);
                let hb = fxhash::hash64(bigram.as_bytes()) as usize;
                v[hb % self.dim] += 1.5;
            }
        }

        // L2 normalize so cosine similarity remains meaningful.
        let norm = v.iter().map(|x| (*x as f64) * (*x as f64)).sum::<f64>().sqrt() as f32;
        if norm > 0.0 {
            for x in &mut v {
                *x /= norm;
            }
        }
        v
    }

    fn embed_with_fastembed(&self, text: &str) -> Result<Vec<f32>, String> {
        let mut model = match &self.backend {
            LocalBackend::FastEmbed(model) => model
                .lock()
                .map_err(|_| "local embedding model lock poisoned".to_owned())?,
            LocalBackend::HashOnly => return Ok(self.hash_embed(text)),
        };

        let vectors = model
            .embed(vec![text], Some(1))
            .map_err(|e| format!("fastembed inference failed: {e}"))?;
        vectors
            .into_iter()
            .next()
            .ok_or_else(|| "fastembed returned empty embedding response".to_owned())
    }
}

impl EmbeddingProvider for LocalEmbeddingProvider {
    fn embedding_dim(&self) -> usize {
        self.dim
    }

    fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
        match self.embed_with_fastembed(text) {
            Ok(v) => Ok(v),
            Err(err) => {
                warn!("Falling back to hash embeddings after local inference error: {err}");
                Ok(self.hash_embed(text))
            }
        }
    }

    fn embed_batch(&self, texts: &[&str], batch_size: usize) -> Result<Vec<Vec<f32>>, String> {
        let chunk_size = batch_size.max(1);
        match &self.backend {
            LocalBackend::FastEmbed(model) => {
                let mut model = model
                    .lock()
                    .map_err(|_| "local embedding model lock poisoned".to_owned())?;
                let mut out = Vec::with_capacity(texts.len());
                for chunk in texts.chunks(chunk_size) {
                    let vectors = model
                        .embed(chunk, Some(chunk_size))
                        .map_err(|e| format!("fastembed inference failed: {e}"))?;
                    out.extend(vectors);
                }
                if out.len() == texts.len() {
                    Ok(out)
                } else {
                    warn!("Falling back to hash embeddings due to fastembed batch size mismatch");
                    Ok(texts.iter().map(|t| self.hash_embed(t)).collect())
                }
            }
            LocalBackend::HashOnly => Ok(texts.iter().map(|t| self.hash_embed(t)).collect()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LocalEmbeddingProvider;
    use crate::ports::embeddings::EmbeddingProvider;

    #[test]
    fn local_embedding_dim_matches_configured_dim() {
        let provider = LocalEmbeddingProvider::new(384);
        assert_eq!(provider.embedding_dim(), 384);
    }

    #[test]
    fn non_empty_text_produces_non_zero_embedding() {
        let provider = LocalEmbeddingProvider::new(384);
        let v = provider.embed("factory method pattern reduces conditional complexity").unwrap();
        assert_eq!(v.len(), provider.embedding_dim());
        let sum_abs: f32 = v.iter().map(|x| x.abs()).sum();
        assert!(sum_abs > 0.0);
    }

    #[test]
    fn semantically_different_texts_produce_different_embeddings() {
        let provider = LocalEmbeddingProvider::new(384);
        let a = provider.embed("dependency inversion and interface boundaries").unwrap();
        let b = provider.embed("recipe for sourdough bread starter hydration").unwrap();
        assert_eq!(a.len(), b.len());
        assert_ne!(a, b);
    }
}
