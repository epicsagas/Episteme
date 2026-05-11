# Packaging de distribution (CLI Rust)

Ce guide explique comment creer une archive de donnees de release pour d'autres utilisateurs avec la CLI Rust.

## Commande

```bash
episteme dist
```

## Ce que `episteme dist` inclut
- `raw/`
- `meta/`
- `data/` (si present)
- `db/episteme.db` (base d'embeddings)

Archive de sortie :
- `dist/episteme-data-<version>.tar.gz`

## Comportement de construction automatique
- Si `~/.episteme/db/episteme.db` est absent, `episteme dist` execute automatiquement `epis build` en premier.
- La base construite est egalement copiee dans le repertoire `db/` local au projet pour inclusion dans l'archive.
- `epis install --local` peuple les donnees a partir de l'archive (ou du repli sur l'arborescence source) et construit automatiquement l'index RAG dans `~/.episteme/`.

## Options
- `--out-dir <REP>` : repertoire de sortie (par defaut : `dist`)
- `--no-db` : ignorer l'inclusion de la base
- `--skip-build` : ne pas construire automatiquement la base si absente

Exemples :

```bash
# packaging par defaut dans dist/
episteme dist

# repertoire de sortie personnalise
episteme dist --out-dir release

# metadata uniquement (sans base)
episteme dist --no-db

# mode strict : echouer si la base est absente
episteme dist --skip-build
```

## Verification
Apres avoir genere l'archive, verifiez la structure :

```bash
tar -tzf dist/episteme-data-*.tar.gz | head -n 30
```

Vous devriez voir les entrees sous :
- `episteme-data-<version>/raw/...`
- `episteme-data-<version>/meta/...`
- `episteme-data-<version>/db/episteme.db` (sauf si `--no-db`)
