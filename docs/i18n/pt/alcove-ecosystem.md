# Ecossistema Alcove — Analise de arquitetura e capacidades

> Uma comparacao detalhada da camada de Conhecimento Tacito do Episteme (TK-*) e do ecossistema de documentacao Alcove, cobrindo modelos de armazenamento, capacidades de busca, gerenciamento do ciclo de vida e orientacao por caso de uso.

---

## 1. Visao geral da arquitetura

### Conhecimento Tacito do Episteme (TK-*)

| Aspecto | Detalhe |
|---------|---------|
| **Armazenamento** | Arquivo SQLite unico (`~/.episteme/user_knowledge.db`) |
| **Schema** | 5 tabelas: `user_entities`, `user_relations`, `user_embeddings`, `user_entities_fts` (virtual FTS5), `insight_seq` |
| **Unidade** | Um insight = uma linha `UserEntity` (ID TK-xxx) |
| **Grafo** | Mesclado com o grafo canonico via `CompositeGraph` em tempo de execucao — permite travessia de caminhos entre camadas (TK-001 → DP-005 → SMELL-01) |
| **Concorrencia** | `Mutex<Connection>` + modo WAL para acesso MCP + CLI simultaneo |

### Sistema de documentacao Alcove

| Aspecto | Detalhe |
|---------|---------|
| **Armazenamento** | Arquivos Markdown no sistema de arquivos + indice Tantivy BM25 + embeddings sqlite-vec |
| **Estrutura** | Classificacao de 3 niveis: Core (7), Suplementar (19), Publico (15) arquivos por projeto |
| **Unidade** | Um arquivo Markdown estruturado (PRD, ARCHITECTURE, DECISIONS, etc.) |
| **Grafo** | Conexoes flexiveis baseadas em wikilinks e caminhos de arquivos |
| **Concorrencia** | Bloqueio baseado em arquivo (`.index_lock`) por raiz de docs, isolamento de indice por vault |
| **Vaults** | 3 links simbolicos para pastas Obsidian PARA: areas (8 docs), resources (71), zettelkasten (17) |

---

## 2. Comparacao de modelos de armazenamento

### Schema TK-* do Episteme

```sql
-- Tabela principal
CREATE TABLE user_entities (
    id TEXT PRIMARY KEY,           -- TK-001, TK-002, ...
    title TEXT,                    -- Auto: primeira linha, max 80 caracteres
    content TEXT,                  -- Texto livre (sem comprimento maximo)
    author TEXT DEFAULT 'user',
    confidence REAL DEFAULT 0.5,   -- +0.05 por link confirmado, limite 1.0
    evidence_count INTEGER DEFAULT 0,
    last_validated TEXT DEFAULT '',
    tags TEXT DEFAULT '[]',        -- Array JSON
    relations TEXT DEFAULT '{}',   -- JSON HashMap<relation_type, Vec<entity_id>>
    link_provenance TEXT DEFAULT '{}',
    created_at TEXT,
    updated_at TEXT
);

-- Relacoes normalizadas (derives_from, applies_to, supersedes)
CREATE TABLE user_relations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_id TEXT,
    relation_type TEXT,
    to_id TEXT,
    UNIQUE(from_id, relation_type, to_id)
);

-- Busca em texto completo FTS5
CREATE VIRTUAL TABLE user_entities_fts USING fts5(title, content, tags, content=user_entities);
```

### Estrutura de arquivos Alcove

```
~/.alcove/
  config.toml                    # Configuracao global (docs_root, listas de arquivos core/team/public, modelo de embedding)
  docs -> link simbolico         # → Obsidian/SecondBrain/99-Archives/projects
  vaults/
    areas -> link simbolico      # → Obsidian/02-Areas (8 docs)
    resources -> link simbolico  # → Obsidian/03-Resources (71 docs)
    zettelkasten -> link simbolico # → Obsidian/10-Zettelkasten (17 docs)
  models/                        # Modelos de embedding ONNX em cache
  logs/

<docs_root>/<project>/
  .alcove/
    index/                       # Arquivos de indice Tantivy BM25
    index_meta.json              # Impressoes digitais de arquivos (mtime + tamanho)
    vectors.db                   # Embeddings sqlite-vec
  PRD.md                         # Requisitos do produto
  ARCHITECTURE.md                # Design do sistema
  PROGRESS.md                    # Marcos e estado
  DECISIONS.md                   # Registros de decisoes arquiteturais
  CONVENTIONS.md                 # Padroes de codificacao
  SECRETS_MAP.md                 # Variaveis de ambiente e secrets
  DEBT.md                        # Registro de divida tecnica
```

---

## 3. Natureza do conhecimento

| Dimensao | Episteme TK-* | Alcove |
|-----------|---------------|--------|
| **Tipo** | Insights momentaneos, licoes aprendidas, decisoes da equipe | Documentacao de projeto estruturada (requisitos, arquitetura, decisoes) |
| **Mutabilidade** | Mutavel (CRUD SQLite) | Mutavel (edicoes de arquivos + reconstrucao de indice) |
| **Fonte** | Texto livre contribuido pelo usuario | Escrito pelo usuario + gerado por agente a partir de templates |
| **Autoridade** | Observacao pessoal/de equipe | Mandato da equipe / politica organizacional |
| **Granularidade** | Atomica (um insight por entrada) | Seccionado (multiplos ADRs por DECISIONS.md) |
| **Vinculacao** | Detectada automaticamente para entidades canonicas (pontuacao de palavras-chave) | Wikilinks manuais + links markdown |
| **Versionamento** | Nenhum (somente SQLite) | Baseado em Git (arquivo = fonte da verdade) |

### Ciclo de vida de um insight (Episteme TK-*)

```
add_insight(text, tags?, project?, linked_entities?)
  │
  ├── Gerar ID TK-xxx (sequencia atomica)
  ├── detect_canonical_links() — correspondencia de palavras-chave → top 5 entidades canonicas
  │     score >= 0.5 → Link automatico (derives_from)
  │     score < 0.5 → Link sugerido
  ├── Deteccao de duplicatas FTS5 → DuplicateCandidate[]
  ├── Persistir em SQLite + cache em memoria
  └── Retornar: { id, auto_links, suggested_links, duplicates, confidence }

confirm_links(id, accepted[], rejected[])
  │
  ├── Adicionar relacoes derives_from/applies_to
  ├── Atualizar fonte link_provenance para "manual"
  ├── Incrementar confianca (+0.05 por link, limite 1.0)
  └── Persistir atualizacoes

search_insights(query, limit?)
  │
  └── Consulta FTS5 MATCH → resultados classificados
```

### Ciclo de vida de um documento (Alcove)

```
init_project(project_name, project_path?)
  │
  ├── Criar 7 documentos core a partir de templates (PRD, ARCHITECTURE, ...)
  ├── Opcionalmente criar documentos publicos (README, CHANGELOG, ...)
  └── Reconstruir indice de busca

validate_docs()
  │
  ├── Verificar existencia de arquivos obrigatorios
  ├── Verificar marcadores de template (TODO, FIXME)
  ├── Verificar cabecalhos de secao obrigatorios
  ├── Verificar contagens minimas de itens de lista
  └── Retornar: aprovado/aviso/falha por arquivo

lint_project()
  │
  ├── Detectar [[wikilinks]] e links markdown quebrados
  ├── Encontrar arquivos orfaos (nao vinculados a partir de nenhum documento)
  ├── Encontrar marcadores obsoletos (WIP, TODO, FIXME, DRAFT, DEPRECATED)
  └── Encontrar referencias de anos desatualizadas (2+ anos)

audit_project()
  │
  ├── Escanear repo de docs privado para docs obrigatorios ausentes
  ├── Escanear repo de projeto publico para docs internos expostos
  ├── Classificar arquivos em niveis
  └── Retornar: suggested_actions[]
```

---

## 4. Capacidades de busca

| Capacidade | Episteme TK-* | Alcove |
|-----------|---------------|--------|
| **Motor** | FTS5 (correspondencia de palavras-chave) | Tantivy BM25 + similaridade de cosseno sqlite-vec |
| **Fusao** | Nenhuma | RRF (Reciprocal Rank Fusion, k=60) |
| **CJK** | Sem suporte especial | NgramTokenizer (min=2, max=3) |
| **Fragmentacao** | N/A (uma linha = um insight) | Fragmentos de 200-500 caracteres |
| **Incremental** | N/A (tabela unica) | Comparacao de impressoes digitais mtime + tamanho |
| **Busca vetorial** | O schema existe (`user_embeddings`) mas **nao conectado** | Completamente operacional (MultilingualE5Small, 384d) |
| **Escopo** | Banco de dados unico | Por projeto ou global (entre projetos) |
| **Fallback** | Nenhum | Correspondencia de subcadeia grep quando nao ha indice |

---

## 5. Completude de funcionalidades

| Funcionalidade | Episteme TK-* | Alcove |
|----------------|---------------|--------|
| Criar | `add_insight` | `init_project`, edicao de arquivos |
| Ler | `search_insights` (somente busca, sem obter por ID) | `get_doc_file`, `search_project_docs` |
| Atualizar | Nao exposto via MCP | Edicao direta de arquivo + `rebuild_index` |
| Excluir | Nao exposto via MCP | Exclusao de arquivo + `rebuild_index` |
| Validacao | Nenhuma | `validate_docs`, `lint_project` |
| Auditoria | Nenhuma | `audit_project` (separacao publico/privado) |
| Backup | Nenhum | `backup_vault` (instantaneo de commit git) |
| Importar | Nenhum | `promote_document` (Obsidian → repo de docs) |
| Politica | Nenhuma | `policy.toml` com niveis de aplicacao |
| Templates | Nenhum | 7 core + 19 suplementares + 15 publicas |

---

## 6. Sistema de vaults Alcove

Tres vaults, vinculados simbolicamente a estrutura PARA do Obsidian:

| Vault | Alvo | Documentos | Proposito |
|-------|------|------------|-----------|
| `areas` | `02-Areas` | 8 | Areas de dominio: agentes MCP, DevOps, Rust, LLM/RAG, Open Source |
| `resources` | `03-Resources` | 71 | Referencia: AWS, Leis de Engenharia de Software, docs tecnicos |
| `zettelkasten` | `10-Zettelkasten` | 17 | Notas atomicas: arquitetura de IA, BM25, grafos de conhecimento, padroes Rust |

Cada vault possui de forma independente:
- Indice BM25 (Tantivy)
- Base de dados vetorial (sqlite-vec)
- Rastreamento de impressoes digitais de arquivos (`index_meta.json`)
- Isolamento de cache (separados `OnceLock<Mutex<HashMap>>`)

---

## 7. Sistema de configuracao Alcove

### Global: `~/.alcove/config.toml`

```toml
docs_root = "/caminho/para/Obsidian/SecondBrain/99-Archives/projects"

[core]
files = ["PRD.md", "ARCHITECTURE.md", "PROGRESS.md", "DECISIONS.md",
         "CONVENTIONS.md", "SECRETS_MAP.md", "DEBT.md"]

[team]
files = ["ENV_SETUP.md", "ONBOARDING.md", "DATA_MODEL.md", "SCHEMA.md",
         "DEPLOYMENT.md", "RUNBOOK.md", "PLAYBOOK.md", "MONITORING.md", ...]  # 19 arquivos

[public]
files = ["README.md", "CHANGELOG.md", "CONTRIBUTING.md", "SECURITY.md", ...]  # 15 arquivos

[embedding]
model = "MultilingualE5Small"
auto_download = true
enabled = true
```

### Por projeto: `alcove.toml`

Sobrescreve os padroes globais para: `diagram_format`, `core_files`, `team_files`, `public_files`.

### Politica: `policy.toml`

Define:
- Nivel `enforce`: `strict` | `warn` | `off`
- Documentos obrigatorios com cabecalhos de secao e contagens minimas de itens
- Convencoes de nomenclatura (`UPPER_SNAKE`, `lower_snake`, `kebab`, `free`)
- Prioridade: projeto > equipe > padroes integrados

---

## 8. Matriz de decisao por caso de uso

| Situacao | Ferramenta recomendada | Justificativa |
|----------|----------------------|---------------|
| « Registrar uma licao aprendida com um incidente em producao » | **Episteme TK-*** | Vincula automaticamente a smells/leis relevantes para referencias cruzadas futuras |
| « Iniciar documentacao para um novo projeto » | **Alcove** `init_project` | 7 templates core gerados automaticamente |
| « Verificar se ha docs desatualizados » | **Alcove** `lint_project` | Detecta automaticamente WIP/TODO/DEPRECATED/datas desatualizadas |
| « Encontrar o que a equipe decidiu sobre o middleware de auth » | **Alcove** `search_project_docs` | Busca em DECISIONS.md estruturado com BM25 + vetorial |
| « Detectar code smells em um modulo » | **Episteme** `analyze_code` | Deteccao de smells baseada em padroes/regex |
| « Garantir que o PRD tem todas as secoes obrigatorias » | **Alcove** `validate_docs` | Validacao de secoes e contagens de itens baseada em politica |
| « Vincular um insight ao padrao Strategy » | **Episteme** `confirm_links` | Cria uma aresta `derives_from` para a entidade canonica |
| « Importar notas do Obsidian para acesso de agentes » | **Alcove** `promote_document` | Importa para o repo de docs com deteccao automatica de projeto |
| « Encontrar a relacao entre SRP e Extract Class » | **Episteme** `find_path` | Travessia de grafo multi-salto atraves de tipos de entidades |
| « Fazer backup do estado da documentacao do projeto » | **Alcove** `backup_vault` | Instantaneo de commit git com marca de tempo |
| « Auditar docs internos expostos no repo publico » | **Alcove** `audit_project` | Escaneia localizacoes privadas e publicas |
| « Obter sugestoes de refactoring classificadas para o codigo » | **Episteme** `suggest_refactorings` | Pontuacao composta: severidade × esforco × alinhamento de principios |

---

## 9. Papeis complementares

```
Episteme TK-*                     Alcove
"Que principio universal          "O que nossa equipe
 se aplica aqui?"                  decidiu sobre isso?"

 Insight momentaneo ←────────────→ Registro de decisao estruturado
 Vinculacao automatica por         Scaffolding baseado em templates
 palavras-chave                    Busca de docs entre projetos
 Travessia de grafo entre          Analise de docs → deteccao de obsolescencia
 camadas
 Analise de codigo → deteccao
   de smells
```

**Quando ambos estao ativos**: Episteme fornece o "por que" universal (leis, padroes), Alcove fornece o "o que decidimos" especifico do projeto (ADR, convencoes). Os agentes devem citar ambas as fontes, com Alcove tendo precedencia quando as regras da equipe entram em conflito com a orientacao generica.

---

## 10. Escala e desempenho

| Metrica | Episteme TK-* | Alcove |
|---------|---------------|--------|
| **Capacidade projetada** | Centenas de insights | ~10.000 arquivos |
| **Latencia de busca** | FTS5 instantaneo (em memoria) | BM25 < 500ms para visao geral |
| **Eficiencia de tokens** | Um insight por resultado | Top-5 fragmentos ~1.5k tokens (vs ~8k para grep) |
| **Reconstrucao de indice** | Nao necessaria (disparadores FTS5) | Incremental: somente arquivos modificados |
| **Tamanho do modelo** | N/A (nao conectado) | 15MB (ArcticEmbedXS) a 2.3GB (BGE-M3) |

---

*Veja tambem: [Guia de integracao Alcove](./alcove-integration.md) para padroes de uso e exemplos de fluxos de trabalho.*
