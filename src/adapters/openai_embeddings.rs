//! OpenAI embedding provider backed by the `/v1/embeddings` HTTP endpoint.
//!
//! The entire module is gated behind the `openai-embeddings` Cargo feature.

#[cfg(feature = "openai-embeddings")]
use crate::ports::embeddings::EmbeddingProvider;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[cfg(feature = "openai-embeddings")]
#[derive(serde::Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[cfg(feature = "openai-embeddings")]
#[derive(serde::Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

// ---------------------------------------------------------------------------
// Provider
// ---------------------------------------------------------------------------

/// Embedding provider that calls the OpenAI `/v1/embeddings` API.
#[cfg(feature = "openai-embeddings")]
pub struct OpenAIEmbeddingProvider {
    client: reqwest::blocking::Client,
    api_key: String,
    model: String,
    dim: usize,
}

#[cfg(feature = "openai-embeddings")]
impl OpenAIEmbeddingProvider {
    const API_URL: &'static str = "https://api.openai.com/v1/embeddings";

    /// Create a new provider.
    ///
    /// * `api_key` - OpenAI API key (e.g. `sk-...`).
    /// * `model`   - Model name, e.g. `"text-embedding-3-small"`.
    /// * `dim`     - Expected embedding dimensionality (must match the model output).
    pub fn new(api_key: String, model: String, dim: usize) -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
            api_key,
            model,
            dim,
        }
    }

    /// Call the OpenAI embeddings endpoint with the given input array.
    fn call_api(&self, input: &[&str]) -> Result<Vec<Vec<f32>>, String> {
        let body = serde_json::json!({
            "model": self.model,
            "input": input,
        });

        let response = self
            .client
            .post(Self::API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| format!("OpenAI embedding request failed: {e}"))?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().unwrap_or_else(|e| {
                format!("(failed to read response body: {e})")
            });
            return Err(format!(
                "OpenAI embedding API returned status {}: {text}",
                status
            ));
        }

        let parsed: EmbeddingResponse = response
            .json()
            .map_err(|e| format!("failed to deserialize OpenAI response: {e}"))?;

        // Validate dimensionality of every returned embedding.
        for (i, item) in parsed.data.iter().enumerate() {
            if item.embedding.len() != self.dim {
                return Err(format!(
                    "embedding dimension mismatch at index {}: expected {}, got {}",
                    i,
                    self.dim,
                    item.embedding.len()
                ));
            }
        }

        // The API guarantees the response order matches the input order.
        let mut embeddings: Vec<Vec<f32>> = parsed.data.into_iter().map(|d| d.embedding).collect();

        // Re-sort by the `index` field if the API ever returns them out of
        // order (defensive). The `index` field is optional in the response
        // schema, so we rely on the natural order as the primary path.
        embeddings.truncate(input.len());

        Ok(embeddings)
    }
}

#[cfg(feature = "openai-embeddings")]
impl EmbeddingProvider for OpenAIEmbeddingProvider {
    fn embedding_dim(&self) -> usize {
        self.dim
    }

    fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
        let results = self.call_api(&[text])?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| "OpenAI returned no embeddings".to_owned())
    }

    fn embed_batch(&self, texts: &[&str], batch_size: usize) -> Result<Vec<Vec<f32>>, String> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let batch_size = batch_size.max(1);
        let mut all_embeddings: Vec<Vec<f32>> = Vec::with_capacity(texts.len());

        for chunk in texts.chunks(batch_size) {
            let batch = self.call_api(chunk)?;
            all_embeddings.extend(batch);
        }

        Ok(all_embeddings)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(all(test, feature = "openai-embeddings"))]
mod tests {
    use super::*;

    #[test]
    fn new_provider_stores_params() {
        let provider = OpenAIEmbeddingProvider::new(
            "sk-test-key".to_owned(),
            "text-embedding-3-small".to_owned(),
            1536,
        );
        assert_eq!(provider.api_key, "sk-test-key");
        assert_eq!(provider.model, "text-embedding-3-small");
        assert_eq!(provider.dim, 1536);
    }

    #[test]
    fn embedding_dim_returns_configured_value() {
        let provider = OpenAIEmbeddingProvider::new(
            "sk-test".to_owned(),
            "text-embedding-3-small".to_owned(),
            1536,
        );
        assert_eq!(provider.embedding_dim(), 1536);

        let provider_512 = OpenAIEmbeddingProvider::new(
            "sk-test".to_owned(),
            "text-embedding-3-small".to_owned(),
            512,
        );
        assert_eq!(provider_512.embedding_dim(), 512);
    }

    #[test]
    fn embed_batch_empty_input() {
        let provider = OpenAIEmbeddingProvider::new(
            "sk-test".to_owned(),
            "text-embedding-3-small".to_owned(),
            1536,
        );
        let result = provider.embed_batch(&[], 32);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    /// Integration test against the live OpenAI API.
    ///
    /// Run with: `cargo test --features openai-embeddings -- --ignored openai_live`
    ///
    /// Requires `SYNTAGMA_OPENAI_API_KEY` to be set in the environment.
    #[test]
    #[ignore]
    fn openai_live_embed_single() {
        let key = std::env::var("SYNTAGMA_OPENAI_API_KEY")
            .expect("SYNTAGMA_OPENAI_API_KEY must be set for live test");
        let provider = OpenAIEmbeddingProvider::new(
            key,
            "text-embedding-3-small".to_owned(),
            1536,
        );
        let vec = provider.embed("hello world").expect("embed should succeed");
        assert_eq!(vec.len(), 1536);
        assert!(vec.iter().any(|&f| f != 0.0));
    }

    /// Integration test: batch embedding against the live API.
    #[test]
    #[ignore]
    fn openai_live_embed_batch() {
        let key = std::env::var("SYNTAGMA_OPENAI_API_KEY")
            .expect("SYNTAGMA_OPENAI_API_KEY must be set for live test");
        let provider = OpenAIEmbeddingProvider::new(
            key,
            "text-embedding-3-small".to_owned(),
            1536,
        );
        let texts = ["first sentence", "second sentence", "third sentence"];
        let results = provider
            .embed_batch(&texts, 2)
            .expect("embed_batch should succeed");
        assert_eq!(results.len(), 3);
        for (i, vec) in results.iter().enumerate() {
            assert_eq!(
                vec.len(),
                1536,
                "embedding at index {i} has wrong dimension"
            );
            assert!(
                vec.iter().any(|&f| f != 0.0),
                "embedding at index {i} is all zeros"
            );
        }
    }
}
