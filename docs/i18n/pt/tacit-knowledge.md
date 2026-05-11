# Arquitetura do conhecimento tacito

O Episteme gerencia duas camadas distintas de conhecimento: **canonico** (imutavel, curado) e **tacito** (mutavel, contribuido por usuarios). Este documento descreve a arquitetura de dois bancos de dados, o fluxo de dados e o ciclo de vida dos insights.

## Visao geral

| | Conhecimento canonico | Conhecimento tacito (Insights) |
|---|---|---|
| **Armazenamento** | `~/.episteme/db/episteme.db` | `~/.episteme/user_knowledge.db` |
| **Mutabilidade** | Somente leitura (reconstruido via `epis build`) | Leitura-escrita (tempo real via MCP) |
| **Prefixo de ID** | `DP-NNN`, `RF-NNN`, `LAW-NNN`, `SMELL-NNN` | `TK-NNN` |
| **Fonte** | Arquivos markdown curados em `raw/` | Ferramenta MCP `add_insight` / CLI `epis insight` |
| **Entidades** | 22 padroes, 66 refactorings, 56 leis, 23 smells | Insights de usuario ilimitados |

Esses dois bancos de dados estao fisicamente separados, mas sao mesclados em tempo de execucao em um unico grafo transitavel.

## Design de dois bancos de dados

```
┌─────────────────────────────────┐     ┌──────────────────────────────┐
│  Base canonica (episteme.db)    │     │  Base de conhecimento        │
│                                 │     │  de usuario                  │
│  ┌───────────┐  ┌────────────┐  │     │  (user_knowledge.db)         │
│  │  chunks   │  │ embeddings │  │     │  ┌────────────────────────┐  │
│  │  (914)    │  │  (914)     │  │     │  │  user_entities         │  │
│  └───────────┘  └────────────┘  │     │  │  (entradas TK-xxx)     │  │
│                                 │     │  ├────────────────────────┤  │
│  Construido por: epis build     │     │  │  user_relations        │  │
│  Povoado de: raw/*.md           │     │  ├────────────────────────┤  │
│                                 │     │  │  user_embeddings       │  │
│  Imutavel em tempo de           │     │  ├────────────────────────┤  │
│  execucao                       │     │  │  user_entities_fts     │  │
│                                 │     │  │  (indice de busca      │  │
└──────────────┬──────────────────┘     │  │   FTS5)                │  │
               │                        │  ├────────────────────────┤  │
               │                        │  │  insight_seq           │  │
               │                        │  │  (contador ID atomico) │  │
               │                        │  └────────────────────────┘  │
               │                        │                              │
               │                        │  Escrito por: MCP add_insight│
               │                        │  Lido por: search_insights   │
               │                        └──────────────┬───────────────┘
               │                                       │
               └───────────────┬───────────────────────┘
                               │
                    ┌──────────▼──────────┐
                    │   CompositeGraph    │
                    │   (mesclagem em     │
                    │    memoria)         │
                    │                     │
                    │  - Busca de         │
                    │    entidade unificada│
                    │  - BFS entre camadas│
                    │  - Consultas de     │
                    │    vizinhos inter-  │
                    │    camadas          │
                    │                     │
                    │  Atende todas as    │
                    │    solicitacoes de  │
                    │    ferramentas MCP  │
                    └─────────────────────┘
```

### Por que bancos de dados separados?

1. **Protecao** — A entrada do usuario nao pode corromper o conhecimento canonico curado.
2. **Ciclo de vida independente** — O conhecimento canonico e atualizado via pipeline de build; o conhecimento tacito e atualizado em tempo real.
3. **Portabilidade** — Compartilhe `user_knowledge.db` entre maquinas ou equipes sem tocar na camada canonica.

## CompositeGraph

A struct `CompositeGraph` (em `src/domain/composite_graph.rs`) mescla ambas as camadas em uma unica interface `GraphRepository` na inicializacao:

- Carrega o `KnowledgeGraph` canonico a partir de `relations.json`
- Abre `user_knowledge.db` via `UserGraphStore`
- Fornece metodos `get_entity()`, `get_neighbors()`, `find_path()` unificados entre ambas as camadas
- Operacoes do usuario nunca modificam o grafo canonico

### Fallback elegante

Se `user_knowledge.db` nao puder ser aberto (arquivo ausente, erro de permissao), o sistema recua para o modo somente canonico. As 6 ferramentas MCP canonicas continuam funcionando; as 3 ferramentas de conhecimento tacito retornam um erro.

## Schema do conhecimento do usuario

```sql
-- Tabela principal de entidades
CREATE TABLE user_entities (
    id TEXT PRIMARY KEY,                    -- ex: "TK-001"
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    author TEXT NOT NULL DEFAULT 'user',
    confidence REAL NOT NULL DEFAULT 0.5,   -- 0.0 a 1.0
    evidence_count INTEGER NOT NULL DEFAULT 0,
    last_validated TEXT NOT NULL DEFAULT '',
    tags TEXT NOT NULL DEFAULT '[]',        -- array JSON
    relations TEXT NOT NULL DEFAULT '{}',   -- JSON: tipo -> [target_ids]
    created_at TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL DEFAULT '',
    link_provenance TEXT NOT NULL DEFAULT '{}'  -- JSON: entity_id -> metadados
);

-- Arestas de relacoes explicitas
CREATE TABLE user_relations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_id TEXT NOT NULL,
    relation_type TEXT NOT NULL,
    to_id TEXT NOT NULL,
    UNIQUE(from_id, relation_type, to_id)
);

-- Vetores de embeddings (f32, little-endian)
CREATE TABLE user_embeddings (
    entity_id TEXT PRIMARY KEY,
    embedding BLOB NOT NULL
);

-- Indice de busca em texto completo
CREATE VIRTUAL TABLE user_entities_fts USING fts5(
    title, content, tags,
    content=user_entities, content_rowid=rowid
);

-- Sequencia ID atomica
CREATE TABLE insight_seq (key TEXT PRIMARY KEY, val INTEGER NOT NULL);
```

## Ferramentas MCP

### add_insight

Cria uma entidade `TK-NNN` a partir de texto livre. O sistema automaticamente:

1. **Detecta links para entidades canonicas** — Correspondencia de palavras-chave em duas fases (filtragem de stop words + pontuacao composta) encontra padroes, leis e smells relevantes.
2. **Verifica duplicatas** — Compara com insights existentes.
3. **Cria relacoes `derives_from`** — Para links de alta confianca (score >= 0.5), vincula automaticamente a entidades canonicas.
4. **Computa correlacoes** — Encontra insights relacionados usando similaridade Jaccard.

Parametros:
- `text` (obrigatorio) — Conteudo do insight em texto livre
- `project` (opcional) — Tag de nome do projeto
- `tags` (opcional) — Tags de categoria
- `linked_entities` (opcional) — IDs de entidades explicitas para vincular (ex: `["DP-005", "SMELL-01"]`)

### search_insights

Busca por palavras-chave FTS5 em insights contribuidos por usuarios. Retorna entidades `TK-*` correspondentes com seu conteudo e relacoes.

Parametros:
- `query` (obrigatorio) — Consulta de busca em linguagem natural
- `limit` (opcional) — Maximo de resultados (padrao 10, maximo 20)

### confirm_links

Valida ou rejeita links detectados automaticamente entre um insight e entidades canonicas. Cada confirmacao:

- Incrementa a pontuacao de confianca do insight (+0.05 por link confirmado, limite 1.0)
- Registra a procedencia do link (fonte, pontuacao, marca de tempo)
- Suporta relacoes merge/supersede entre insights

Parametros:
- `insight_id` (obrigatorio) — O ID `TK-NNN`
- `accepted` (obrigatorio) — IDs de entidades para confirmar como links validos
- `rejected` (opcional) — IDs de entidades para rejeitar
- `merged_with` (opcional) — ID do insight alvo para fusao/substituicao

## Ciclo de vida de um insight

```
1. add_insight("마이크로서비스 분리 시 도메인 경계를 먼저 식별하기로 결정")
       │
       ▼
2. Deteccao automatica de links: CONWAY-001 (Lei de Conway), DP-026 (Strangler Fig)
       │
       ▼
3. Criar TK-001 com derives_from → LAW-017, DP-026
       │
       ▼
4. confirm_links(insight_id="TK-001", accepted=["LAW-017"])
       │
       ▼
5. Confianca incrementada: 0.5 → 0.55
       │
       ▼
6. Mais tarde: search_insights("마이크로서비스 분리") → retorna TK-001
       │
       ▼
7. find_path("TK-001", "SMELL-03") → atravessa o grafo entre camadas
```

## Tipos de relacoes

| Relacao | Direcao | Descricao |
|----------|-----------|-------------|
| `derives_from` | TK → Canonico | Insight fundamentado em uma entidade canonica |
| `applies_to` | TK → Canonico | Insight que aplica um padrao/lei a um contexto especifico |
| `supersedes` | TK → TK | Um insight mais recente substitui um anterior |
| `related_to` | TK → TK/Canonico | Conexao semantica geral |

## Uso via CLI

```bash
# Adicionar um insight
epis insight add "팀에서 God Class 리팩토링 시 Extract Class보다 Facade Pattern이 효과적이었음"

# Buscar insights
epis insight search "인증 미들웨어"

# Listar todos os insights
epis insight list
```

## Arquivos-fonte principais

| Arquivo | Funcao |
|---------|--------|
| `src/domain/composite_graph.rs` | Mesclagem em tempo de execucao das camadas canonica + usuario |
| `src/adapters/user_graph_store.rs` | `MutableGraphRepository` respaldado por SQLite |
| `src/server/mcp_insight.rs` | Handlers MCP para as 3 ferramentas de conhecimento tacito |
| `src/adapters/insight_utils.rs` | Geracao de IDs, marcas de tempo, utilidades de texto |
| `src/domain/types.rs` | `UserEntity`, `LinkProvenance`, `EntityType::Insight` |
| `src/ports/graph.rs` | Trait `MutableGraphRepository` (14 metodos) |
