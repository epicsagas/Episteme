"""
Centralized configuration for Syntagma.
All paths, constants, and environment variables in one place.
"""

import os
from pathlib import Path
from typing import Any

# ===== Home directory =====
# Override with SYNTAGMA_HOME env var for custom installations.
SYNTAGMA_HOME = Path(os.getenv("SYNTAGMA_HOME", Path.home() / ".syntagma"))


def _load_yaml_config() -> dict[str, Any]:
    """Load ~/.syntagma/config.yaml if it exists. Returns empty dict on any failure."""
    config_path = SYNTAGMA_HOME / "config.yaml"
    if not config_path.exists():
        return {}
    try:
        import yaml  # type: ignore[import-untyped]

        return yaml.safe_load(config_path.read_text(encoding="utf-8")) or {}
    except Exception:
        return {}


def _cfg(yaml: dict[str, Any], section: str, key: str, env_var: str, default: Any) -> Any:
    """Resolve a config value: env var > config.yaml > default."""
    env_val = os.getenv(env_var)
    if env_val is not None:
        return env_val
    return yaml.get(section, {}).get(key, default)


_yaml = _load_yaml_config()

# ===== Runtime paths (all under SYNTAGMA_HOME) =====
DATA_DIR = SYNTAGMA_HOME / "data"  # relations, code_smells, file_to_entity
DB_DIR = SYNTAGMA_HOME / "db"  # vector database
RAW_DIR = SYNTAGMA_HOME / "raw"  # raw knowledge markdown files
LOG_DIR = SYNTAGMA_HOME / "logs"  # service logs
CACHE_DIR = SYNTAGMA_HOME / "cache"  # local on-disk cache (future use)

DB_PATH = DB_DIR / "syntagma.db"
RELATIONS_PATH = DATA_DIR / "relations.json"
CODE_SMELLS_PATH = DATA_DIR / "code_smells.json"
FILE_TO_ENTITY_PATH = DATA_DIR / "file_to_entity.json"

# BASE_DIR kept as an alias so existing callers (KnowledgeGraph, RAG builder,
# CodeSmellDetector) that do base_dir / "meta" and base_dir / "raw" still work
# after data migration.  Points to SYNTAGMA_HOME; sub-paths map as:
#   <base_dir>/meta  -> DATA_DIR  (symlinked by `syntagma install --init-data`)
#   <base_dir>/raw   -> RAW_DIR
BASE_DIR = SYNTAGMA_HOME

# ===== Service paths =====
PID_FILE = SYNTAGMA_HOME / "mcp.pid"

# ===== Repo source dir (used only during `syntagma install --init-data`) =====
# This is the location of the package source; only valid in editable / repo installs.
_PACKAGE_DIR = Path(__file__).parent
_REPO_ROOT = _PACKAGE_DIR.parent.parent  # src/syntagma -> src -> repo root

# ===== Entity Types =====
ENTITY_TYPES = ["pattern", "refactoring", "law", "smell"]
ENTITY_PREFIXES = {
    "pattern": "DP-",
    "refactoring": "RF-",
    "law": "LAW-",
    "smell": "SMELL-",
}

# ===== Categories (7 software engineering domains) =====
CATEGORIES = {
    1: "teams",
    2: "planning",
    3: "architecture",
    4: "quality",
    5: "scalability",
    6: "design",
    7: "decisions",
}

# ===== Search =====
SIMILARITY_THRESHOLD = 0.5
DEFAULT_SEARCH_LIMIT = 5
MAX_SEARCH_LIMIT = 20

# ===== Token Budgets =====
MAX_TOKENS_PER_RESPONSE = 500

# ===== API Server =====
API_HOST = os.getenv("UVICORN_HOST", "0.0.0.0")
API_PORT = int(os.getenv("UVICORN_PORT", "8000"))
API_KEYS = os.getenv("SYNTAGMA_API_KEYS", "").strip()
LOG_LEVEL = os.getenv("LOG_LEVEL", "INFO")
ENABLE_JSON_LOGGING = os.getenv("ENABLE_JSON_LOGGING", "true").lower() == "true"
ENABLE_DEBUG_ENDPOINTS = os.getenv("ENABLE_DEBUG_ENDPOINTS", "false").lower() == "true"

# ===== MCP Server =====
MCP_SERVER_HOST = os.getenv("SYNTAGMA_MCP_HOST", "localhost")
MCP_SERVER_PORT = int(os.getenv("SYNTAGMA_MCP_PORT", "43175"))

# ===== Redis Cache =====
REDIS_HOST = _cfg(_yaml, "redis", "host", "SYNTAGMA_REDIS_HOST", "localhost")
REDIS_PORT = int(_cfg(_yaml, "redis", "port", "SYNTAGMA_REDIS_PORT", 6379))
REDIS_DB = int(_cfg(_yaml, "redis", "db", "SYNTAGMA_REDIS_DB", 0))
REDIS_TTL = int(_cfg(_yaml, "redis", "ttl", "SYNTAGMA_REDIS_TTL", 3600))
REDIS_ENABLED = (
    str(_cfg(_yaml, "redis", "enabled", "SYNTAGMA_REDIS_ENABLED", "true")).lower() == "true"
)

# ===== Embedding =====
EMBEDDING_MODEL = "all-MiniLM-L6-v2"
EMBEDDING_DIMENSIONS = 384

# ===== Embedding Provider =====
EMBEDDING_PROVIDER = os.getenv("SYNTAGMA_EMBEDDING_PROVIDER", "local")  # "local" | "openai"
OPENAI_API_KEY = os.getenv("OPENAI_API_KEY", "")
OPENAI_EMBED_MODEL = os.getenv("SYNTAGMA_OPENAI_EMBED_MODEL", "text-embedding-3-small")
OPENAI_EMBED_DIM = 1536
