# Documentacao da API REST Episteme

**Versao:** 0.1.0
**URL base:** `http://localhost:8000`

---

## Inicio rapido

```bash
# Iniciar servidor
epis api

# Ou com host/porta personalizados
epis api --host 0.0.0.0 --port 8000

# Docker
docker-compose up -d
```

---

## Autenticacao

Todos os endpoints exceto `/`, `/health`, `/live`, `/ready` requerem autenticacao por chave de API.

### Autenticacao por chave de API

**Cabecalho:** `X-API-Key: <sua-chave-api>`

**Modos:**

1. **Modo Producao** - Definir a variavel de ambiente `EPISTEME_API_KEYS`
   - Lista separada por virgulas de chaves de API validas
   - Todos os endpoints protegidos requerem uma chave valida
   - Retorna 401 Unauthorized se ausente/invalida

2. **Modo Desenvolvimento** - Deixar `EPISTEME_API_KEYS` vazio ou indefinido
   - Nenhuma autenticacao requerida

### Gerar chaves de API

```bash
openssl rand -base64 32
```

### Exemplos de requisicoes

```bash
# Com autenticacao (producao)
curl -X POST http://localhost:8000/analyze \
  -H "Content-Type: application/json" \
  -H "X-API-Key: sua-chave-api" \
  -d '{"code": "def long_method(): pass", "min_confidence": 0.5}'

# Sem autenticacao (modo dev)
curl -X POST http://localhost:8000/analyze \
  -H "Content-Type: application/json" \
  -d '{"code": "def long_method(): pass"}'
```

---

## Limitacao de taxa

Todos os endpoints possuem limitacao de taxa por endereco IP com evacucao de buckets baseada em TTL.

| Endpoint | Limite de taxa | Motivo |
|----------|----------------|--------|
| `/analyze` | 20/minuto | Intensivo em CPU |
| `/refactor` | 20/minuto | Intensivo em CPU |
| `/search` | 50/minuto | Computacao de embeddings |
| `/stats`, `/graph/*` | 100/minuto | Padrao |
| `/`, `/health` | Ilimitado | Publico |

Quando excedido, retorna 429 com o cabecalho `Retry-After`.

---

## Endpoints

### Saude e informacoes

#### `GET /`

Informacoes do servico.

**Resposta:**
```json
{
  "name": "episteme",
  "version": "0.1.0",
  "description": "Software engineering knowledge graph",
  "endpoints": ["analyze", "search", "graph", "refactor", "stats"]
}
```

#### `GET /health`

Verificacao de saude com estado dos componentes.

**Resposta:**
```json
{
  "status": "healthy",
  "components": {
    "knowledge_graph": "ok",
    "rag_database": "ok",
    "embedding_provider": "local"
  }
}
```

#### `GET /live`

Sonda de vivacidade: `{"status": "alive"}`

#### `GET /ready`

Sonda de prontidao: `{"status": "ready"}` (503 se nao estiver pronto)

#### `GET /stats`

Estatisticas do grafo.

**Resposta:**
```json
{
  "total_entities": 161,
  "total_edges": 201,
  "by_type": {
    "refactoring": 66,
    "law": 56,
    "pattern": 22,
    "smell": 17
  }
}
```

---

### Analise de codigo

#### Code smells suportados (16 detectores)

| ID | Nome | Linguagens |
|---|---|---|
| SMELL-01 | Long Method | Todas |
| SMELL-02 | Long Parameter List | Todas |
| SMELL-03 | Primitive Obsession | Python |
| SMELL-04 | Large Class | Todas |
| SMELL-05 | Data Clumps | Todas (stub) |
| SMELL-06 | Switch Statements | Todas |
| SMELL-07 | Data Class | Todas |
| SMELL-09 | Shotgun Surgery | Todas (stub) |
| SMELL-10 | Divergent Change | Todas |
| SMELL-11 | Lazy Class | Todas |
| SMELL-12 | Speculative Generality | Todas |
| SMELL-13 | Duplicate Code | Todas (parcial) |
| SMELL-14 | Middle Man | Todas |
| SMELL-18 | Feature Envy | Todas |
| SMELL-20 | Message Chains | Todas |
| SMELL-21 | God Object | Todas |

#### `POST /analyze`

Detectar code smells.

**Requisicao:**
```json
{
  "code": "def long_method():\n    ...",
  "language": "python",
  "min_confidence": 0.5
}
```

**Resposta:**
```json
{
  "count": 2,
  "smells": [
    {
      "smell_id": "SMELL-01",
      "smell_name": "Long Method",
      "confidence": 0.90,
      "location": "temp.py:1",
      "function_name": "long_method",
      "metrics": {
        "loc": 94,
        "cyclomatic_complexity": 27,
        "nesting_depth": 5,
        "parameter_count": 9
      },
      "reasons": ["LOC=94 exceeds 30", "CC=27 exceeds 10"]
    }
  ]
}
```

#### `POST /refactor`

Obter sugestoes de refactoring classificadas para os smells detectados.

**Requisicao:**
```json
{
  "code": "def long_method():\n    ...",
  "top_k": 3,
  "min_confidence": 0.5
}
```

**Resposta:**
```json
{
  "count": 1,
  "analyses": [
    {
      "smell": { "smell_id": "SMELL-01", "smell_name": "Long Method" },
      "suggestions": [
        {
          "refactoring_id": "RF-001",
          "title": "Extract Method",
          "priority_score": 0.79,
          "effort": "medium",
          "principles_enforced": ["LAW-040", "LAW-042-S"]
        }
      ]
    }
  ]
}
```

---

### Busca

#### `GET /search`

Busca via parametro de consulta: `/search?q=strategy+pattern&top_k=5`

#### `POST /search`

Busca semantica na base de conhecimento.

**Requisicao:**
```json
{
  "query": "How to fix Long Method?",
  "top_k": 5,
  "entity_type": "refactoring"
}
```

**Resposta:**
```json
{
  "count": 3,
  "results": [
    {
      "entity_id": "RF-001",
      "title": "Extract Method",
      "category": "refactoring",
      "similarity": 0.85,
      "content": "Extract Method is a refactoring technique..."
    }
  ]
}
```

---

### Grafo de conhecimento

#### `GET /graph/{id}`

Obter detalhes de uma entidade pelo seu ID.

**Exemplo:** `GET /graph/DP-005`

#### `GET /graph/{id}/neighbors`

Obter vizinhos de uma entidade: `/graph/SMELL-01/neighbors?relation_type=solved_by`

#### `POST /graph/neighbors`

Obter vizinhos (POST).

**Requisicao:**
```json
{
  "entity_id": "SMELL-01",
  "relation_type": "solved_by"
}
```

#### `GET /graph/path`

Caminho mais curto: `/graph/path?from_id=SMELL-01&to_id=LAW-042-S&max_depth=5`

#### `POST /graph/subgraph`

Extrair subgrafo.

**Requisicao:**
```json
{
  "entity_id": "DP-005",
  "depth": 2
}
```

#### `GET /graph/contradictions`

Encontrar entidades com relacoes conflitantes.

#### `POST /graph/infer-transitive`

Inferir relacoes de aplicacao transitiva.

---

### Monitoramento

#### `GET /metrics`

Metricas no formato Prometheus incluindo:
- `http_requests_total` — por metodo, endpoint, estado
- `episteme_smells_detected_total` — por smell_id
- `episteme_searches_total` — por entity_type
- `episteme_analysis_duration_seconds` — histograma

---

## Desempenho

| Endpoint | Latencia media | Notas |
|----------|---------------|-------|
| `/analyze` | ~5ms | Parsing regex + AST (cache OnceLock) |
| `/refactor` | ~10ms | Inclui travessia do grafo |
| `/search` | ~20ms | FTS5 + similaridade de cosseno |
| `/graph/neighbors` | ~1ms | Grafo em memoria |
| `/graph/path` | ~5ms | BFS ate profundidade 5 |

---

## Tratamento de erros

| Estado | Significado |
|--------|-------------|
| 200 | Sucesso |
| 400 | Requisicao invalida |
| 401 | Chave de API ausente/invalida |
| 404 | Entidade nao encontrada |
| 429 | Limite de taxa excedido |
| 500 | Erro interno |

---

## Variaveis de ambiente

```bash
# Servidor
EPISTEME_API_HOST=0.0.0.0
EPISTEME_API_PORT=8000
EPISTEME_API_KEYS=key1,key2

# Dados
EPISTEME_DATA_DIR=~/.episteme/data
EPISTEME_DB_PATH=~/.episteme/db/episteme.db

# Registros
RUST_LOG=info
```

---

## Licenca

Licenca APACHE-2.0
