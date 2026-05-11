# Guia de desenvolvimento do Episteme

**Projeto:** Episteme v0.1.0
**Linguagem:** Rust (edicao 2024)
**Ultima atualizacao:** 2026-05-03

---

## Status atual

| Componente | Status | Detalhes |
|-------------|--------|----------|
| **Base de conhecimento** | Completo | 22 padroes, 66 refactorings, 56 leis, 23 smells, 201 relacoes |
| **Deteccao de code smells** | Producao | 16 funcoes detectoras, 10 linguagens |
| **API REST** | Producao | 17 endpoints (axum), limitacao de taxa, autenticacao |
| **Servidor MCP** | Producao | 6 ferramentas, transporte stdio + HTTP |
| **Pipeline RAG** | Producao | SQLite + FTS5 + fastembed (ONNX) |
| **Visualizacao do grafo** | Producao | UI web interativa com D3-force |

---

## Arquitetura

Arquitetura hexagonal (ports & adapters):

```
src/
├── commands/          # Handlers de subcomandos CLI (clap)
│   ├── analysis.rs    # analyze, infer
│   ├── build.rs       # build (pipeline RAG)
│   ├── explore.rs     # explore (busca/REPL)
│   ├── graph.rs       # consultas de grafo
│   ├── install.rs     # assistente de instalacao (TUI)
│   ├── service.rs     # gerenciamento do daemon MCP HTTP
│   └── other.rs       # api, mcp, web, telemetry, hooks
├── adapters/          # Camada de infraestrutura
│   ├── regex_parsers.rs   # GenericParser (10 linguagens, cache regex OnceLock)
│   ├── python_ast_parser.rs  # AST Python (rustpython-parser)
│   ├── search_engines.rs  # Palavras-chave FTS5 + similaridade de cosseno
│   ├── service.rs         # Daemon MCP HTTP
│   ├── sqlite_db.rs       # Pool de conexoes SQLite
│   ├── cache.rs           # Cache Redis (opcional)
│   └── ...
├── domain/            # Logica de negocio (sem dependencias externas)
│   ├── graph.rs       # KnowledgeGraph (BFS, subgrafo, contradicoes, Jaccard)
│   ├── detectors.rs   # 16 detectores de smells com TieredAccum
│   ├── engine.rs      # RefactoringInferenceEngine + RefactoringRanker
│   ├── summarizer.rs  # Otimizacao de resposta por nivel de detalhe
│   └── types.rs       # EntityType, RelationType, tipos centrais
├── server/            # Camada HTTP (axum)
│   ├── api_routes.rs  # 17 endpoints REST
│   ├── mcp_handler.rs # Fachada fina MCP
│   ├── mcp_search.rs  # Servico de busca
│   ├── mcp_graph.rs   # Servico de grafo
│   └── mcp_analysis.rs # Servico de analise de codigo
└── ports/             # Traits (limites hexagonais)
    ├── parser.rs      # Trait CodeParser
    ├── search.rs      # Trait SearchEngine
    ├── graph.rs       # Trait GraphStore
    └── embeddings.rs  # Trait EmbeddingProvider
```

---

## Stack tecnologica

| Componente | Tecnologia | Proposito |
|-------------|-----------|-----------|
| **Linguagem** | Rust (edicao 2024) | Seguranca, desempenho, binario unico |
| **Framework web** | axum | API REST + transporte HTTP MCP |
| **Banco de dados** | rusqlite (SQLite integrado) | Grafo de conhecimento + armazenamento vetorial |
| **Busca** | FTS5 + similaridade de cosseno | Busca hibrida por palavras-chave e semantica |
| **Embeddings** | fastembed (ONNX Runtime) | Geracao de embeddings local, sem configuracao |
| **CLI** | clap (derive) | 15 subcomandos |
| **AST Python** | rustpython-parser | Deteccao de smells Python baseada em AST |
| **Outras linguagens** | regex (cache OnceLock) | Framework GenericParser |

---

## Detectores de code smells (16)

| ID | Smell | Deteccao |
|----|-------|----------|
| SMELL-01 | Long Method | Limiar de LOC |
| SMELL-02 | Long Parameter List | Contagem de parametros |
| SMELL-03 | Primitive Obsession | Proporcao de parametros primitivos |
| SMELL-04 | Large Class | Contagem de metodos + campos |
| SMELL-05 | Data Clumps | Grupos de parametros repetidos (stub) |
| SMELL-06 | Switch Statements | Contagem de switch/match |
| SMELL-07 | Data Class | Razao metodos vs campos |
| SMELL-08 | Temporary Field | Uso condicional de campos (stub) |
| SMELL-09 | Shotgun Surgery | Acoplamento de mudancas (stub) |
| SMELL-10 | Divergent Change | Metricas de coesao de metodos |
| SMELL-11 | Lazy Class | Baixo LOC + contagem de metodos |
| SMELL-12 | Speculative Generality | Abstrato sem concreto |
| SMELL-13 | Duplicate Code | Similaridade baseada em hash (parcial) |
| SMELL-14 | Middle Man | Razao de delegacao |
| SMELL-15 | Parallel Inheritance Hierarchies | Espelhamento de hierarquia (stub) |
| SMELL-16 | Comments | Razao comentarios/codigo (stub) |
| SMELL-17 | Dead Code | Deteccao de inalcancavel/nao usado (stub) |
| SMELL-18 | Feature Envy | Razao de chamadas externas |
| SMELL-19 | Inappropriate Intimacy | Acesso privado entre classes (stub) |
| SMELL-20 | Message Chains | Profundidade da cadeia de chamadas |
| SMELL-21 | God Object | Composto: LOC + metodos + acoplamento |
| SMELL-22 | Refused Bequest | Razao override-para-nada (stub) |
| SMELL-23 | Alternative Classes with Different Interfaces | Divergencia de interface (stub) |

---

## Configuracao de desenvolvimento

```bash
# Clonar e compilar (requer Rust 1.95+)
git clone https://github.com/epicsagas/Episteme.git
cd Episteme
cargo build

# Executar testes
cargo test

# Lint
cargo clippy -- -D warnings

# Instalar localmente (prepara dados e constroi DB automaticamente)
cargo install --path .
epis install --local
```

---

## Endpoints da API (17)

| Metodo | Caminho | Descricao |
|--------|---------|-----------|
| GET | `/` | Informacoes do servico |
| GET | `/health` | Verificacao de saude |
| GET | `/live` | Sonda de vivacidade |
| GET | `/ready` | Sonda de prontidao |
| GET | `/stats` | Estatisticas do grafo |
| POST | `/analyze` | Deteccao de code smells |
| POST | `/refactor` | Sugestoes de refactoring |
| GET | `/search` | Busca no conhecimento |
| POST | `/search` | Busca no conhecimento (POST) |
| GET | `/graph/{id}` | Obter entidade |
| GET | `/graph/{id}/neighbors` | Obter vizinhos |
| POST | `/graph/neighbors` | Obter vizinhos (POST) |
| POST | `/graph/subgraph` | Extrair subgrafo |
| GET | `/graph/path` | Caminho mais curto |
| GET | `/graph/contradictions` | Encontrar contradicoes |
| POST | `/graph/infer-transitive` | Inferir relacoes transitivas |
| GET | `/metrics` | Metricas Prometheus |

---

## Roadmap futuro

- **Plugins IDE** — Integracoes nativas para VSCode, IntelliJ
- **Entidades personalizadas** — Adicionar padroes/smells especificos da equipe
- **Metricas de equipe** — Agregar uso de padroes na organizacao
- **Documentacao multilingue** — Base de conhecimento em Coreano, Japones, Chines
- **Tutoriais interativos** - Tour guiado no aplicativo para ferramentas MCP

---

*Ultima atualizacao: 2026-05-03*
