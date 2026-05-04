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
- `meta/syntagma.db` (compatibility copy)

Output archive:
- `dist/syntagma-data-<version>.tar.gz`

## Auto-build behavior
- If `~/.syntagma/db/syntagma.db` is missing, `syntagma dist` automatically runs `syntagma build` first.
- This makes packaging work on fresh machines without pre-built DBs.

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
