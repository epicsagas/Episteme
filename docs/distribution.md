# Distribution Packaging (Rust CLI)

This guide explains how to create a release data archive for other users with the Rust CLI.

## Command

```bash
episteme dist
```

## What `episteme dist` includes
- `raw/`
- `meta/`
- `data/` (if present)
- `db/episteme.db` (embedding DB)

Output archive:
- `dist/episteme-data-<version>.tar.gz`

## Auto-build behavior
- If `~/.episteme/db/episteme.db` is missing, `episteme dist` automatically runs `epis build` first.
- The built DB is also copied to the project-local `db/` directory for inclusion in the archive.
- `epis install --local` seeds data from the archive (or source tree fallback) and auto-builds the RAG index to `~/.episteme/`.

## Options
- `--out-dir <DIR>`: output directory (default: `dist`)
- `--no-db`: skip DB inclusion
- `--skip-build`: do not auto-build DB if missing

Examples:

```bash
# default packaging to dist/
episteme dist

# custom output directory
episteme dist --out-dir release

# package metadata only (no DB)
episteme dist --no-db

# strict mode: fail if DB missing
episteme dist --skip-build
```

## Verification
After generating the archive, verify structure:

```bash
tar -tzf dist/episteme-data-*.tar.gz | head -n 30
```

You should see entries under:
- `episteme-data-<version>/raw/...`
- `episteme-data-<version>/meta/...`
- `episteme-data-<version>/db/episteme.db` (unless `--no-db`)
