#!/usr/bin/env bash
# Build the knowledge data asset for a GitHub Release.
#
# Usage:
#   ./scripts/package_data.sh [VERSION]
#
# Output:
#   dist/syntagma-data-<VERSION>.tar.gz
#
# The archive contains:
#   meta/   ← JSON knowledge files + syntagma.db (vector database)
#   raw/    ← Markdown source documents
#
# This archive is uploaded as a GitHub Release asset and downloaded by
# `syntagma install` to seed ~/.syntagma/.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION="${1:-$(python3 -c "import tomllib; d=tomllib.load(open('pyproject.toml','rb')); print(d['project']['version'])" 2>/dev/null || grep '^version' pyproject.toml | head -1 | tr -d ' ' | cut -d'"' -f2)}"
DIST_DIR="$REPO_ROOT/dist"
ARCHIVE_NAME="syntagma-data-${VERSION}.tar.gz"
ARCHIVE_PATH="$DIST_DIR/$ARCHIVE_NAME"
DB_IN_META="$REPO_ROOT/meta/syntagma.db"

mkdir -p "$DIST_DIR"

echo "Packaging Syntagma knowledge data v${VERSION}..."

# ── Generate vector DB if not already present ─────────────────────────────────
if [ ! -f "$DB_IN_META" ]; then
    echo "Building vector database (syntagma build)..."
    pip install --quiet -e "$REPO_ROOT[dev]" 2>/dev/null || pip install --quiet -e "$REPO_ROOT"
    syntagma install --local
    syntagma build
    cp "$HOME/.syntagma/db/syntagma.db" "$DB_IN_META"
    echo "Vector DB built and copied to meta/syntagma.db"
fi

# Build tar from repo root, excluding hidden files
tar -czf "$ARCHIVE_PATH" \
    -C "$REPO_ROOT" \
    --exclude="*/.DS_Store" \
    --exclude="*/._*" \
    meta/ \
    raw/

# Remove temporary DB copy from meta/
rm -f "$DB_IN_META"

SIZE=$(du -sh "$ARCHIVE_PATH" | cut -f1)
echo "Created: $ARCHIVE_PATH ($SIZE)"
echo ""
echo "Upload this file to the GitHub Release for v${VERSION}:"
echo "  gh release upload v${VERSION} $ARCHIVE_PATH"
