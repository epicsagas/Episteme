# Empaquetado de distribucion (CLI Rust)

Esta guia explica como crear un archivo de datos de release para otros usuarios con la CLI Rust.

## Comando

```bash
episteme dist
```

## Que incluye `episteme dist`
- `raw/`
- `meta/`
- `data/` (si esta presente)
- `db/episteme.db` (base de embeddings)

Archivo de salida:
- `dist/episteme-data-<version>.tar.gz`

## Comportamiento de construccion automatica
- Si `~/.episteme/db/episteme.db` falta, `episteme dist` ejecuta automaticamente `epis build` primero.
- La base construida tambien se copia al directorio `db/` local del proyecto para inclusion en el archivo.
- `epis install --local` pobla datos desde el archivo (o fallback del arbol de fuentes) y construye automaticamente el indice RAG en `~/.episteme/`.

## Opciones
- `--out-dir <DIR>`: directorio de salida (por defecto: `dist`)
- `--no-db`: omitir inclusion de la base
- `--skip-build`: no construir automaticamente la base si falta

Ejemplos:

```bash
# empaquetado por defecto en dist/
episteme dist

# directorio de salida personalizado
episteme dist --out-dir release

# solo metadatos (sin base)
episteme dist --no-db

# modo estricto: fallar si la base falta
episteme dist --skip-build
```

## Verificacion
Despues de generar el archivo, verifique la estructura:

```bash
tar -tzf dist/episteme-data-*.tar.gz | head -n 30
```

Deberia ver entradas bajo:
- `episteme-data-<version>/raw/...`
- `episteme-data-<version>/meta/...`
- `episteme-data-<version>/db/episteme.db` (a menos que `--no-db`)
