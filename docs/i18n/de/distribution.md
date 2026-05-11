# Distributionsverpackung (Rust-CLI)

Dieser Leitfaden erklärt, wie Sie mit der Rust-CLI ein Release-Datenarchiv für andere Benutzer erstellen.

## Befehl

```bash
episteme dist
```

## Was `episteme dist` einschließt
- `raw/`
- `meta/`
- `data/` (falls vorhanden)
- `db/episteme.db` (Embedding-DB)

Ausgabearchiv:
- `dist/episteme-data-<version>.tar.gz`

## Auto-Build-Verhalten
- Wenn `~/.episteme/db/episteme.db` fehlt, führt `episteme dist` automatisch zuerst `epis build` aus.
- Die erstellte DB wird ebenfalls in das projektlokale `db/`-Verzeichnis kopiert, um in das Archiv aufgenommen zu werden.
- `epis install --local` importiert Daten aus dem Archiv (oder Quellbaum-Fallback) und erstellt den RAG-Index automatisch in `~/.episteme/`.

## Optionen
- `--out-dir <DIR>`: Ausgabeverzeichnis (Standard: `dist`)
- `--no-db`: DB-Einbindung überspringen
- `--skip-build`: DB nicht automatisch erstellen, falls fehlend

Beispiele:

```bash
# Standardverpackung in dist/
episteme dist

# Benutzerdefiniertes Ausgabeverzeichnis
episteme dist --out-dir release

# Nur Metadaten verpacken (keine DB)
episteme dist --no-db

# Strenger Modus: fehlschlagen, wenn DB fehlt
episteme dist --skip-build
```

## Überprüfung
Nach dem Erstellen des Archivs überprüfen Sie die Struktur:

```bash
tar -tzf dist/episteme-data-*.tar.gz | head -n 30
```

Sie sollten Einträge unter sehen:
- `episteme-data-<version>/raw/...`
- `episteme-data-<version>/meta/...`
- `episteme-data-<version>/db/episteme.db` (außer bei `--no-db`)
