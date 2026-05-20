use crate::ports::embeddings::EmbeddingProvider;

pub struct NoopEmbeddingProvider {
    dim: usize,
}

impl NoopEmbeddingProvider {
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }
}

impl EmbeddingProvider for NoopEmbeddingProvider {
    fn embedding_dim(&self) -> usize {
        self.dim
    }

    fn embed(&self, _text: &str) -> Result<Vec<f32>, String> {
        Ok(vec![0.0f32; self.dim])
    }

    fn embed_batch(&self, texts: &[&str], _batch_size: usize) -> Result<Vec<Vec<f32>>, String> {
        Ok(texts.iter().map(|_| vec![0.0f32; self.dim]).collect())
    }
}
