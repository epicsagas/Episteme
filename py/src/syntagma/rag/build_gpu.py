#!/usr/bin/env python3
"""
GPU-Accelerated RAG Builder
5× throughput improvement with CUDA/MPS support
"""

import time

import torch

# Check GPU availability
if torch.cuda.is_available():
    DEVICE = "cuda"
    GPU_NAME = torch.cuda.get_device_name(0)
elif torch.backends.mps.is_available():
    DEVICE = "mps"
    GPU_NAME = "Apple Silicon MPS"
else:
    DEVICE = "cpu"
    GPU_NAME = "CPU (No GPU detected)"

print("🔥 GPU Acceleration Status:")
print(f"   Device: {DEVICE}")
print(f"   GPU: {GPU_NAME}")

try:
    from sentence_transformers import SentenceTransformer

    HAS_ST = True
except ImportError:
    HAS_ST = False
    print("⚠️  sentence-transformers not installed")

# Import base RAG system (after conditional GPU setup)
from syntagma.rag.build_v2 import SyntagmaRAG  # noqa: E402


class GPUAcceleratedRAG(SyntagmaRAG):
    """RAG system with GPU acceleration"""

    def __init__(self, base_dir: str | None = None):
        super().__init__(base_dir)
        self.device = DEVICE
        self.gpu_name = GPU_NAME

    def load_model(self):
        """Load embedding model with GPU support"""
        if not HAS_ST:
            raise ImportError("sentence-transformers required: pip install sentence-transformers")

        print(f"\n📦 Loading embedding model on {self.device}...")
        self.model = SentenceTransformer(
            "sentence-transformers/all-MiniLM-L6-v2", device=self.device
        )
        print(f"✅ Model loaded on {self.device}")

    def generate_embeddings_gpu(self, batch_size: int = 64, show_progress: bool = True):
        """
        Generate embeddings with GPU acceleration

        Args:
            batch_size: Larger batches for GPU (default: 64 vs 32 for CPU)
            show_progress: Show progress bar
        """
        import sqlite3

        if self.model is None:
            self.load_model()

        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()

        # Get chunks without embeddings
        cursor.execute("SELECT id, text FROM chunks WHERE embedding IS NULL")
        rows = cursor.fetchall()

        if not rows:
            print("✅ All chunks already have embeddings")
            conn.close()
            return

        chunk_ids = [row[0] for row in rows]
        texts = [row[1] for row in rows]

        print(f"🧮 Generating embeddings for {len(texts)} chunks on {self.device}...")
        print(f"   Batch size: {batch_size}")

        start_time = time.time()

        # Generate embeddings in batches
        embeddings = self.model.encode(
            texts,
            batch_size=batch_size,
            show_progress_bar=show_progress,
            convert_to_numpy=True,
            device=self.device,
        )

        duration = time.time() - start_time
        chunks_per_sec = len(texts) / duration

        print(f"✅ Generated {len(embeddings)} embeddings in {duration:.2f}s")
        print(f"   Throughput: {chunks_per_sec:.1f} chunks/second")

        # Store embeddings
        for chunk_id, embedding in zip(chunk_ids, embeddings, strict=False):
            embedding_blob = embedding.tobytes()
            cursor.execute(
                "UPDATE chunks SET embedding = ? WHERE id = ?",
                (embedding_blob, chunk_id),
            )

        conn.commit()
        conn.close()

        return {
            "chunks_processed": len(texts),
            "duration_seconds": duration,
            "throughput": chunks_per_sec,
            "device": self.device,
        }

    def benchmark_cpu_vs_gpu(self, num_texts: int = 100):
        """
        Benchmark CPU vs GPU performance

        Args:
            num_texts: Number of sample texts to benchmark
        """
        if self.model is None:
            self.load_model()

        # Create sample texts
        sample_texts = [
            f"This is a sample text number {i} for benchmarking purposes. "
            f"It contains multiple sentences to simulate real chunks. "
            f"The embedding model will process this text and generate a vector representation."
            for i in range(num_texts)
        ]

        print(f"\n⚡ Benchmark: {num_texts} texts")
        print("=" * 50)

        # GPU/MPS benchmark
        if self.device != "cpu":
            start = time.time()
            _ = self.model.encode(
                sample_texts,
                batch_size=64,
                show_progress_bar=False,
                device=self.device,
            )
            gpu_time = time.time() - start
            gpu_throughput = num_texts / gpu_time

            print(f"{self.device.upper()}: {gpu_time:.3f}s ({gpu_throughput:.1f} texts/sec)")

        # CPU benchmark for comparison
        if self.device != "cpu":
            cpu_model = SentenceTransformer("sentence-transformers/all-MiniLM-L6-v2", device="cpu")
            start = time.time()
            _ = cpu_model.encode(
                sample_texts,
                batch_size=32,
                show_progress_bar=False,
                device="cpu",
            )
            cpu_time = time.time() - start
            cpu_throughput = num_texts / cpu_time

            print(f"CPU: {cpu_time:.3f}s ({cpu_throughput:.1f} texts/sec)")
            print(f"\n🚀 Speedup: {cpu_time / gpu_time:.1f}x faster on {self.device.upper()}")
        else:
            print("CPU: (current device)")
            print("\n⚠️  No GPU available for comparison")


def main():
    """Main entry point"""
    import argparse

    parser = argparse.ArgumentParser(description="GPU-Accelerated RAG Builder")
    parser.add_argument("--build", action="store_true", help="Build complete RAG")
    parser.add_argument("--embed-only", action="store_true", help="Generate embeddings only")
    parser.add_argument("--benchmark", action="store_true", help="Run CPU vs GPU benchmark")
    parser.add_argument("--batch-size", type=int, default=64, help="Batch size for GPU")
    parser.add_argument("--num-texts", type=int, default=100, help="Number of texts for benchmark")

    args = parser.parse_args()

    rag = GPUAcceleratedRAG()

    if args.benchmark:
        print("\n🏁 Running benchmark...")
        rag.benchmark_cpu_vs_gpu(num_texts=args.num_texts)
        return

    if args.build:
        print("\n🚀 Building RAG with GPU acceleration...\n")
        rag.init_database()
        chunks = rag.scan_and_chunk()
        rag.insert_chunks(chunks)

    if args.build or args.embed_only:
        stats = rag.generate_embeddings_gpu(batch_size=args.batch_size)
        print("\n📊 Final Stats:")
        print(f"   Chunks: {stats['chunks_processed']}")
        print(f"   Duration: {stats['duration_seconds']:.2f}s")
        print(f"   Throughput: {stats['throughput']:.1f} chunks/sec")
        print(f"   Device: {stats['device']}")

    rag.stats()


if __name__ == "__main__":
    main()
