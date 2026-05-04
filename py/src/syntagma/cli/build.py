#!/usr/bin/env python3
"""
syntagma build — generate the vector database from seeded knowledge data.

Reads ~/.syntagma/raw/ (seeded by `syntagma install`), chunks the markdown,
generates sentence-transformer embeddings, and stores everything in
~/.syntagma/db/syntagma.db.

GPU acceleration (CUDA / Apple MPS) is used automatically when available
via the EmbeddingsClient device auto-detection.
"""

from __future__ import annotations

import argparse
import sys

from syntagma import config as _config


def _check_data_ready() -> bool:
    missing = []
    for path in [_config.RELATIONS_PATH, _config.FILE_TO_ENTITY_PATH]:
        if not path.exists():
            missing.append(str(path))
    if missing:
        print("Error: seed data not found. Run `syntagma install` first.", file=sys.stderr)
        for m in missing:
            print(f"  missing: {m}", file=sys.stderr)
        return False
    return True


def _device_from_args(args: argparse.Namespace) -> str | None:
    """Translate --gpu / --no-gpu flags to a device string for EmbeddingsClient."""
    if args.no_gpu:
        return "cpu"
    if args.gpu:
        try:
            import torch
            if torch.cuda.is_available():
                return "cuda"
            if torch.backends.mps.is_available():
                return "mps"
        except ImportError:
            pass
        print("Warning: --gpu requested but no GPU found, using CPU.", file=sys.stderr)
        return "cpu"
    return None  # auto-detect


def cmd_build(args: argparse.Namespace) -> int:
    if not _check_data_ready():
        return 1

    device = _device_from_args(args)

    # Inject device into the global EmbeddingsClient before RAG init
    from syntagma.embeddings import client as _ec
    _ec._client = _ec.EmbeddingsClient(device=device)

    from syntagma.rag.build_v2 import SyntagmaRAG
    rag = SyntagmaRAG(base_dir=None)

    if args.rebuild and _config.DB_PATH.exists():
        print(f"Removing existing database: {_config.DB_PATH}")
        _config.DB_PATH.unlink()

    print("\nBuilding Syntagma knowledge database...")
    print(f"  Source : {_config.RAW_DIR}")
    print(f"  Output : {_config.DB_PATH}\n")

    rag.init_database()
    chunks = rag.scan_and_chunk()
    rag.insert_chunks(chunks)
    rag.generate_embeddings(batch_size=args.batch_size)

    rag.stats()
    print(f"\nDone. Database ready at {_config.DB_PATH}")
    return 0


def cmd_stats(_args: argparse.Namespace) -> int:
    if not _config.DB_PATH.exists():
        print(f"No database found at {_config.DB_PATH}")
        print("Run `syntagma build` to create it.")
        return 1

    from syntagma.rag.build_v2 import SyntagmaRAG
    rag = SyntagmaRAG(base_dir=None)
    rag.stats()
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="syntagma build",
        description=(
            "Generate the Syntagma vector database from seeded knowledge data.\n\n"
            "Reads markdown files from ~/.syntagma/raw/ (seeded by `syntagma install`),\n"
            "chunks them, generates sentence-transformer embeddings, and stores everything\n"
            "in ~/.syntagma/db/syntagma.db for semantic search and MCP tools.\n\n"
            "GPU acceleration (CUDA / Apple MPS) is used automatically when available."
        ),
        epilog=(
            "Examples:\n"
            "  syntagma build               # auto-detect GPU\n"
            "  syntagma build --no-gpu      # force CPU\n"
            "  syntagma build --rebuild     # drop and rebuild from scratch\n"
            "  syntagma build --stats       # show database statistics"
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("--gpu", action="store_true", help="Force GPU mode")
    parser.add_argument("--no-gpu", action="store_true", help="Force CPU mode")
    parser.add_argument(
        "--batch-size",
        type=int,
        default=64,
        help="Embedding batch size (default: 64)",
    )
    parser.add_argument("--rebuild", action="store_true", help="Drop and rebuild from scratch")
    parser.add_argument("--stats", action="store_true", help="Show database statistics and exit")
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    if args.stats:
        return cmd_stats(args)
    return cmd_build(args)


if __name__ == "__main__":
    sys.exit(main())
