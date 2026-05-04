/// Abstraction over embedding backends (local ONNX, OpenAI, etc.).
pub trait EmbeddingProvider: Send + Sync {
    /// Dimensionality of the vectors this provider returns.
    fn embedding_dim(&self) -> usize;

    /// Embed a single text string.
    fn embed(&self, text: &str) -> Result<Vec<f32>, String>;

    /// Embed a batch of texts. `batch_size` controls how many texts are
    /// sent to the backend in a single call (the method internally loops).
    fn embed_batch(&self, texts: &[&str], batch_size: usize) -> Result<Vec<Vec<f32>>, String>;
}
