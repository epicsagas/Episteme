"""Provider-agnostic embedding client for Syntagma."""

from __future__ import annotations

import contextlib
import io
import threading
import warnings

import numpy as np

from syntagma.config import (
    EMBEDDING_DIMENSIONS,
    EMBEDDING_MODEL,
    EMBEDDING_PROVIDER,
    OPENAI_API_KEY,
    OPENAI_EMBED_DIM,
    OPENAI_EMBED_MODEL,
)


class EmbeddingsClient:
    def __init__(self, provider: str | None = None, device: str | None = None):
        self.provider = provider or EMBEDDING_PROVIDER  # "local" | "openai"
        self._device_override = device  # None = auto-detect
        self._local_encoder = None  # lazy
        self._encoder_lock = threading.Lock()

    @property
    def embedding_dim(self) -> int:
        return OPENAI_EMBED_DIM if self.provider == "openai" else EMBEDDING_DIMENSIONS

    def embed(self, text: str) -> np.ndarray:
        if self.provider == "openai":
            return self._embed_openai(text)
        return self._embed_local(text)

    def embed_batch(self, texts: list[str], batch_size: int = 32) -> np.ndarray:
        if self.provider == "openai":
            return self._embed_openai_batch(texts)
        return self._embed_local_batch(texts, batch_size=batch_size)

    def _embed_local(self, text: str) -> np.ndarray:
        encoder = self._get_local_encoder()
        return np.array(encoder.encode(text, normalize_embeddings=True))

    def _embed_local_batch(self, texts: list[str], batch_size: int = 32) -> np.ndarray:
        encoder = self._get_local_encoder()
        return np.array(
            encoder.encode(
                texts,
                normalize_embeddings=True,
                show_progress_bar=True,
                batch_size=batch_size,
            )
        )

    def _resolve_device(self) -> str:
        """Auto-detect best available device unless overridden."""
        if self._device_override:
            return self._device_override
        try:
            with contextlib.redirect_stderr(io.StringIO()):
                import torch
        except (ImportError, RuntimeError):
            return "cpu"
        try:
            cuda = torch.cuda.is_available()
            mps = torch.backends.mps.is_available()
            if cuda:
                return "cuda"
            if mps:
                return "mps"
        except RuntimeError as e:
            warnings.warn(
                f"GPU initialization failed, falling back to CPU: {e}", RuntimeWarning, stacklevel=2
            )
        return "cpu"

    def _get_local_encoder(self):
        if self._local_encoder is None:
            with self._encoder_lock:
                if self._local_encoder is None:
                    try:
                        from sentence_transformers import SentenceTransformer
                    except ImportError as exc:
                        raise ImportError(
                            "sentence-transformers가 설치되지 않았습니다. "
                            "pip install sentence-transformers 를 실행하세요."
                        ) from exc
                    device = self._resolve_device()
                    print(f"📦 Loading embedding model: {EMBEDDING_MODEL} (device: {device})...")
                    self._local_encoder = SentenceTransformer(EMBEDDING_MODEL, device=device)
        return self._local_encoder

    def _embed_openai(self, text: str) -> np.ndarray:
        if not OPENAI_API_KEY:
            raise ValueError(
                "OPENAI_API_KEY 환경변수가 설정되지 않았습니다. "
                "export OPENAI_API_KEY=<your-key> 를 실행하세요."
            )
        try:
            from openai import OpenAI
        except ImportError as exc:
            raise ImportError(
                "openai 패키지가 설치되지 않았습니다. pip install openai 를 실행하세요."
            ) from exc
        client = OpenAI(api_key=OPENAI_API_KEY)
        response = client.embeddings.create(model=OPENAI_EMBED_MODEL, input=[text])
        return np.array(response.data[0].embedding, dtype=np.float32)

    def _embed_openai_batch(self, texts: list[str]) -> np.ndarray:
        if not OPENAI_API_KEY:
            raise ValueError(
                "OPENAI_API_KEY 환경변수가 설정되지 않았습니다. "
                "export OPENAI_API_KEY=<your-key> 를 실행하세요."
            )
        try:
            from openai import OpenAI
        except ImportError as exc:
            raise ImportError(
                "openai 패키지가 설치되지 않았습니다. pip install openai 를 실행하세요."
            ) from exc
        client = OpenAI(api_key=OPENAI_API_KEY)
        response = client.embeddings.create(model=OPENAI_EMBED_MODEL, input=texts)
        return np.array([d.embedding for d in response.data], dtype=np.float32)


_client: EmbeddingsClient | None = None


def get_client(device: str | None = None) -> EmbeddingsClient:
    global _client
    # device 힌트가 명시된 경우 새 인스턴스 반환 (build --gpu/--no-gpu 대응)
    if device is not None:
        return EmbeddingsClient(device=device)
    if _client is None:
        _client = EmbeddingsClient()
    return _client
