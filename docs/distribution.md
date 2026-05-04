# Distribution Packaging (Rust CLI)

This guide explains how to create a release data archive for other users with the Rust CLI.

## Command

```bash
syntagma dist
```

## What `syntagma dist` includes
- `raw/`
- `meta/`
- `data/` (if present)
- `db/syntagma.db` (embedding DB)

Output archive:
- `dist/syntagma-data-<version>.tar.gz`

## Auto-build behavior
- If `~/.syntagma/db/syntagma.db` is missing, `syntagma dist` automatically runs `syntagma build` first.
- The built DB is also copied to the project-local `db/` directory for inclusion in the archive.
- `syntagma install --local` seeds data from the archive (or source tree fallback) and auto-builds the RAG index to `~/.syntagma/`.

## Options
- `--out-dir <DIR>`: output directory (default: `dist`)
- `--no-db`: skip DB inclusion
- `--skip-build`: do not auto-build DB if missing

Examples:

```bash
# default packaging to dist/
syntagma dist

# custom output directory
syntagma dist --out-dir release

# package metadata only (no DB)
syntagma dist --no-db

# strict mode: fail if DB missing
syntagma dist --skip-build
```

## Verification
After generating the archive, verify structure:

```bash
tar -tzf dist/syntagma-data-*.tar.gz | head -n 30
```

You should see entries under:
- `syntagma-data-<version>/raw/...`
- `syntagma-data-<version>/meta/...`
- `syntagma-data-<version>/db/syntagma.db` (unless `--no-db`)
