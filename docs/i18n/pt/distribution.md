# Empacotamento de distribuicao (CLI Rust)

Este guia explica como criar um pacote de dados de release para outros usuarios com a CLI Rust.

## Comando

```bash
episteme dist
```

## O que o `episteme dist` inclui
- `raw/`
- `meta/`
- `data/` (se presente)
- `db/episteme.db` (base de embeddings)

Arquivo de saida:
- `dist/episteme-data-<version>.tar.gz`

## Comportamento de build automatico
- Se `~/.episteme/db/episteme.db` estiver ausente, `episteme dist` executa `epis build` automaticamente primeiro.
- A base construida tambem e copiada para o diretorio `db/` local do projeto para inclusao no arquivo.
- `epis install --local` prepara dados a partir do arquivo (ou fallback da arvore de fontes) e constroi automaticamente o indice RAG em `~/.episteme/`.

## Opcoes
- `--out-dir <DIR>`: diretorio de saida (padrao: `dist`)
- `--no-db`: pular inclusao da base
- `--skip-build`: nao construir a base automaticamente se estiver ausente

Exemplos:

```bash
# empacotamento padrao em dist/
episteme dist

# diretorio de saida personalizado
episteme dist --out-dir release

# apenas metadados (sem base)
episteme dist --no-db

# modo estrito: falhar se a base estiver ausente
episteme dist --skip-build
```

## Verificacao
Apos gerar o arquivo, verifique a estrutura:

```bash
tar -tzf dist/episteme-data-*.tar.gz | head -n 30
```

Voce devera ver entradas em:
- `episteme-data-<version>/raw/...`
- `episteme-data-<version>/meta/...`
- `episteme-data-<version>/db/episteme.db` (a menos que `--no-db`)
