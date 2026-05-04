import numpy as np
import pytest
from unittest.mock import MagicMock, patch


# --- local provider ---


def test_local_embed_returns_ndarray():
    mock_encoder = MagicMock()
    mock_encoder.encode.return_value = np.zeros(384, dtype=np.float32)
    mock_st_module = MagicMock()
    mock_st_module.SentenceTransformer.return_value = mock_encoder
    with patch.dict("sys.modules", {"sentence_transformers": mock_st_module}):
        from syntagma.embeddings.client import EmbeddingsClient

        client = EmbeddingsClient(provider="local")
        client._local_encoder = None  # force lazy init
        result = client.embed("hello")
    assert isinstance(result, np.ndarray)
    assert result.shape == (384,)


def test_local_embed_batch_returns_2d():
    mock_encoder = MagicMock()
    mock_encoder.encode.return_value = np.zeros((3, 384), dtype=np.float32)
    mock_st_module = MagicMock()
    mock_st_module.SentenceTransformer.return_value = mock_encoder
    with patch.dict("sys.modules", {"sentence_transformers": mock_st_module}):
        from syntagma.embeddings.client import EmbeddingsClient

        client = EmbeddingsClient(provider="local")
        client._local_encoder = None  # force lazy init
        result = client.embed_batch(["a", "b", "c"])
    assert isinstance(result, np.ndarray)
    assert result.shape == (3, 384)


def test_local_missing_package_raises_import_error():
    with patch.dict("sys.modules", {"sentence_transformers": None}):
        from syntagma.embeddings import client as m
        import importlib

        importlib.reload(m)
        c = m.EmbeddingsClient(provider="local")
        c._local_encoder = None
        with pytest.raises(ImportError, match="sentence-transformers"):
            c._get_local_encoder()


# --- openai provider ---


def test_openai_missing_key_raises_value_error():
    from syntagma.embeddings.client import EmbeddingsClient

    client = EmbeddingsClient(provider="openai")
    with patch("syntagma.embeddings.client.OPENAI_API_KEY", ""):
        with pytest.raises(ValueError, match="OPENAI_API_KEY"):
            client._embed_openai("hello")


def test_openai_missing_package_raises_import_error():
    from syntagma.embeddings.client import EmbeddingsClient

    client = EmbeddingsClient(provider="openai")
    with patch("syntagma.embeddings.client.OPENAI_API_KEY", "sk-test"):
        with patch.dict("sys.modules", {"openai": None}):
            with pytest.raises(ImportError, match="openai"):
                client._embed_openai("hello")


def test_openai_embed_returns_ndarray():
    mock_response = MagicMock()
    mock_response.data[0].embedding = [0.1] * 1536
    mock_openai_cls = MagicMock()
    mock_openai_cls.return_value.embeddings.create.return_value = mock_response

    from syntagma.embeddings.client import EmbeddingsClient

    client = EmbeddingsClient(provider="openai")
    with patch("syntagma.embeddings.client.OPENAI_API_KEY", "sk-test"):
        with patch("syntagma.embeddings.client.OpenAI", mock_openai_cls, create=True):
            # patch the import inside the method
            with patch.dict("sys.modules", {"openai": MagicMock(OpenAI=mock_openai_cls)}):
                result = client._embed_openai("hello")
    assert isinstance(result, np.ndarray)
    assert result.shape == (1536,)


# --- _resolve_device ---


def test_resolve_device_override():
    from syntagma.embeddings.client import EmbeddingsClient

    client = EmbeddingsClient(provider="local", device="cpu")
    assert client._resolve_device() == "cpu"


def test_resolve_device_no_torch_returns_cpu():
    import syntagma.embeddings.client as _mod
    from syntagma.embeddings.client import EmbeddingsClient

    client = EmbeddingsClient(provider="local")
    # Patch the module-level contextlib so redirect_stderr triggers ImportError
    # without touching sys.modules["torch"] (which causes C-extension reinit crashes).
    original = _mod.contextlib
    try:
        import contextlib as _ctx

        class _FakeCtx:
            class redirect_stderr:
                def __init__(self, *a):
                    pass

                def __enter__(self):
                    raise ImportError("No module named 'torch'")

                def __exit__(self, *a):
                    return False

        _mod.contextlib = _FakeCtx()
        result = client._resolve_device()
    finally:
        _mod.contextlib = original
    assert result == "cpu"


def test_resolve_device_runtime_error_warns_and_returns_cpu():
    try:
        import torch
    except RuntimeError:
        pytest.skip("torch module in corrupted state due to MLX/torch docstring conflict")
    from syntagma.embeddings.client import EmbeddingsClient

    client = EmbeddingsClient(provider="local")

    with patch.object(torch.cuda, "is_available", side_effect=RuntimeError("CUDA init failed")):
        with pytest.warns(RuntimeWarning, match="GPU initialization failed"):
            result = client._resolve_device()
    assert result == "cpu"


def test_resolve_device_stderr_not_leaked():
    """redirect_stderr must not permanently replace sys.stderr."""
    import sys
    from syntagma.embeddings.client import EmbeddingsClient

    original_stderr = sys.stderr
    client = EmbeddingsClient(provider="local")
    with patch.dict("sys.modules", {"torch": None}):
        client._resolve_device()
    assert sys.stderr is original_stderr


# --- OpenAI batch ---


def test_embed_batch_openai_uses_single_call():
    """embed_batch for openai must call the API once, not N times."""
    mock_response = MagicMock()
    mock_response.data = [MagicMock(embedding=[0.1] * 1536) for _ in range(3)]
    mock_openai_cls = MagicMock()
    mock_openai_cls.return_value.embeddings.create.return_value = mock_response

    from syntagma.embeddings.client import EmbeddingsClient

    client = EmbeddingsClient(provider="openai")
    with patch("syntagma.embeddings.client.OPENAI_API_KEY", "sk-test"):
        with patch.dict("sys.modules", {"openai": MagicMock(OpenAI=mock_openai_cls)}):
            result = client.embed_batch(["a", "b", "c"])

    assert result.shape == (3, 1536)
    assert mock_openai_cls.return_value.embeddings.create.call_count == 1


# --- _get_local_encoder thread safety ---


def test_get_local_encoder_thread_safe():
    """Concurrent calls must initialize the encoder only once."""
    import threading as _threading

    mock_encoder = MagicMock()
    mock_st_module = MagicMock()
    mock_st_module.SentenceTransformer.return_value = mock_encoder

    with patch.dict("sys.modules", {"sentence_transformers": mock_st_module}):
        from syntagma.embeddings import client as m
        import importlib

        importlib.reload(m)

        c = m.EmbeddingsClient(provider="local")

        results = []

        def call():
            results.append(c._get_local_encoder())

        threads = [_threading.Thread(target=call) for _ in range(5)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

    assert mock_st_module.SentenceTransformer.call_count == 1
    assert all(r is results[0] for r in results)


# --- embedding_dim ---


def test_embedding_dim_local():
    from syntagma.embeddings.client import EmbeddingsClient

    assert EmbeddingsClient(provider="local").embedding_dim == 384


def test_embedding_dim_openai():
    from syntagma.embeddings.client import EmbeddingsClient

    assert EmbeddingsClient(provider="openai").embedding_dim == 1536
