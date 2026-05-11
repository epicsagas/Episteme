# Guia de integracao MCP

> Integre o grafo de conhecimento do Episteme no Claude Code, Cursor e outras ferramentas de IA compativeis com MCP

## Modo HTTP MCP Rust (Atual)
Use o transporte HTTP standalone diretamente:

```bash
# Iniciar MCP sobre HTTP
episteme mcp --http --host 127.0.0.1 --port 43175
```

Comportamento de autenticacao:
- Se `EPISTEME_API_KEYS` esta configurado, as requisicoes devem incluir:
```http
Authorization: Bearer <api-key>
```
- Se nenhuma chave esta configurada, a autenticacao e ignorada (modo desenvolvimento).
- `GET /health` e sempre publico para verificacoes de saude.

Nota:
- `epis service` gerencia este mesmo modo HTTP MCP em segundo plano (`start|stop|status|enable|disable`).
- Exemplos antigos com `--proxy` estao obsoletos; use `mcp --http`/`service` diretamente.

## O que e MCP?

[Model Context Protocol (MCP)](https://modelcontextprotocol.io) e um padrao aberto que permite a assistentes de IA acessar ferramentas e fontes de dados externas. O Episteme fornece 6 ferramentas MCP que dao aos agentes de IA acesso direto ao conhecimento de engenharia de software.

---

## Inicio rapido (Claude Code)

### 1. Instalar Episteme

```bash
# Instalar (requer Rust 1.95+)
cargo install --git https://github.com/epicsagas/Episteme

# Instalar agentes e servidor MCP no Claude Code
# (prepara dados e configura MCP automaticamente)
epis install claude
```

> Se o download de dados falhar, use a instalacao via codigo-fonte: `git clone` → `cargo build --release` → `epis install --local`

### 2. Verificar a instalacao

Verifique `~/.claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "episteme": {
      "command": "episteme",
      "args": ["mcp"]
    }
  }
}
```

### 3. Comecar a usar

Reinicie o Claude Code. Voce agora tem acesso a 6 ferramentas do Episteme:

```
Usuario: "Qual a melhor maneira de corrigir um smell God Object?"

Claude (usando a ferramenta search_knowledge):
  → Busca refactorings de "God Object"
  → Retorna: RF-018 (Extract Class), RF-023 (Move Method)
  
Claude: "O anti-padrao God Object (SMELL-03) viola o Principio da 
Responsabilidade Unica (LAW-001). Melhores refactorings:

1. Extract Class (RF-018) - Mover metodos/campos relacionados para nova classe
2. Move Method (RF-023) - Realocar metodos para as classes apropriadas

Ambos aplicam os principios SOLID e melhoram a testabilidade."
```

---

## Referencia de ferramentas MCP

### 1. `search_knowledge`

**Proposito**: Busca semantica em todas as entidades (padroes, leis, refactorings, smells)

**Parametros**:
```typescript
{
  query: string          // Consulta em linguagem natural
  top_k?: number         // Resultados a retornar (padrao: 5)
  filter_type?: string   // "pattern", "law", "refactoring", "smell"
}
```

**Retorna**:
```typescript
{
  results: [{
    entity_id: string     // ex: "DP-023"
    title: string
    entity_type: string
    similarity: number    // 0.0-1.0
    summary: string
  }]
}
```

**Exemplo de conversa**:
```
Usuario: "Como faco meu codigo mais testavel?"

Claude chama: search_knowledge({
  query: "improve testability",
  top_k: 3
})

Retorna:
- LAW-001: Single Responsibility Principle
- DP-018: Dependency Injection
- RF-042: Extract Interface

Claude: "Tres abordagens-chave para melhorar a testabilidade:
1. Aplicar SRP (LAW-001) - Uma classe, uma razao para mudar
2. Usar Dependency Injection (DP-023) - Injetar dependencias
3. Extract Interface (RF-042) - Simular dependencias externas"
```

---

### 2. `get_entity`

**Proposito**: Obter detalhes completos de uma entidade especifica por ID

**Parametros**:
```typescript
{
  entity_id: string   // ex: "DP-023", "RF-001", "SMELL-01"
}
```

**Retorna**:
```typescript
{
  entity_id: string
  title: string
  type: string
  description: string
  implementation: string    // Exemplos de codigo
  when_to_use: string
  benefits: string[]
  trade_offs: string[]
  related_entities: {
    relation_type: string
    target_id: string
    description: string
  }[]
}
```

**Exemplo de conversa**:
```
Usuario: "Explique Strategy Pattern em detalhes"

Claude chama: get_entity({ entity_id: "DP-023" })

Retorna:
- Descricao completa de Strategy Pattern
- Quando usar (multiplos algoritmos, troca em tempo de execucao)
- Implementacao (Context, interface Strategy, ConcreteStrategies)
- Beneficios: Principio Open/Closed, flexibilidade em tempo de execucao
- Compensacoes: Maior numero de classes
- Relacionados: LAW-002 (Open/Closed), RF-034 (Replace Conditional with Polymorphism)

Claude: "Strategy Pattern (DP-023) encapsula algoritmos em classes separadas...
[fornece explicacao detalhada com exemplo de codigo]

Conceitos relacionados:
- Aplica o Principio Open/Closed (LAW-002)
- Frequentemente aplicado via Replace Conditional with Polymorphism (RF-034)"
```

---

### 3. `get_neighbors`

**Proposito**: Explorar entidades conectadas a uma entidade dada

**Parametros**:
```typescript
{
  entity_id: string
  relation_type?: string   // "solves", "enforces", "violates", "relates_to"
  max_depth?: number       // Padrao: 1
}
```

**Retorna**:
```typescript
{
  neighbors: [{
    entity_id: string
    title: string
    relation_type: string
    distance: number
  }]
}
```

**Exemplo de conversa**:
```
Usuario: "Quais refactorings resolvem o smell Long Method?"

Claude chama: get_neighbors({
  entity_id: "SMELL-01",
  relation_type: "solved_by"
})

Retorna:
- RF-001: Extract Method
- RF-034: Decompose Conditional
- RF-056: Replace Temp with Query

Claude: "Long Method (SMELL-01) pode ser resolvido com:
1. Extract Method (RF-001) - Dividir em metodos menores
2. Decompose Conditional (RF-034) - Simplificar condicoes complexas
3. Replace Temp with Query (RF-056) - Eliminar variaveis temporarias"
```

---

### 4. `find_path`

**Proposito**: Encontrar a conexao entre duas entidades (ex: como uma lei se relaciona com um padrao)

**Parametros**:
```typescript
{
  from_id: string
  to_id: string
  max_depth?: number   // Padrao: 5
}
```

**Retorna**:
```typescript
{
  path: [{
    entity_id: string
    title: string
    relation_to_next: string
  }],
  path_found: boolean
  hops: number
}
```

**Exemplo de conversa**:
```
Usuario: "Como o Principio da Responsabilidade Unica se relaciona com o refactoring Extract Class?"

Claude chama: find_path({
  from_id: "LAW-001",
  to_id: "RF-018"
})

Retorna o caminho:
LAW-001 (SRP) --enforces--> DP-007 (Facade) --achieved_via--> RF-018 (Extract Class)

Claude: "SRP (LAW-001) se conecta com Extract Class (RF-018) em 2 saltos:
1. SRP aplica o Padrao Facade (DP-007)
2. Facade frequentemente e alcancado via Extract Class (RF-018)

Insight: Quando uma classe tem multiplas responsabilidades, extrair 
classes (RF-018) ajuda a satisfazer o SRP (LAW-001)."
```

---

### 5. `analyze_code`

**Proposito**: Detectar code smells via analise AST

**Parametros**:
```typescript
{
  code: string
  language: string   // "python", "java", "typescript", etc.
  min_confidence?: number   // Padrao: 0.5
}
```

**Retorna**:
```typescript
{
  smells_detected: number,
  detections: [{
    smell_id: string
    smell_name: string
    confidence: number
    location: string
    metrics: {
      loc: number
      cyclomatic_complexity: number
      parameter_count: number
    }
  }]
}
```

**Exemplo de conversa**:
```
Usuario: "Revise este codigo de validacao de pagamentos"
[colar codigo]

Claude chama: analyze_code({
  code: "...",
  language: "python"
})

Retorna:
- SMELL-01: Long Method (confianca 0.87, LOC=45)
- SMELL-08: Long Parameter List (confianca 0.92, params=9)

Claude: "Encontrei 2 code smells:

1. Long Method (SMELL-01, confianca 87%)
   - 45 linhas de codigo (limite: 20)
   - Alta complexidade ciclomatica (12)
   - Recomendado: Extract Method (RF-001)

2. Long Parameter List (SMELL-08, confianca 92%)
   - 9 parametros (limite: 4)
   - Recomendado: Introduce Parameter Object (RF-029)"
```

---

### 6. `suggest_refactorings`

**Proposito**: Obter sugestoes de refactoring classificadas para os smells detectados

**Parametros**:
```typescript
{
  code: string
  language: string
  top_k?: number   // Numero de sugestoes (padrao: 3)
  min_confidence?: number
}
```

**Retorna**:
```typescript
{
  smells_analyzed: number,
  results: [{
    smell: {
      smell_id: string
      smell_name: string
      confidence: number
    },
    suggestions: [{
      refactoring_id: string
      title: string
      priority_score: number
      severity: string
      effort: string
      principles: string[]
    }]
  }]
}
```

**Exemplo de conversa**:
```
Usuario: "O que eu deveria refatorar neste codigo?"
[colar classe grande]

Claude chama: suggest_refactorings({
  code: "...",
  language: "java",
  top_k: 3
})

Retorna sugestoes classificadas para God Object smell:
1. RF-018: Extract Class (prioridade 0.89, severidade Alta, esforco Medio)
2. RF-023: Move Method (prioridade 0.76, severidade Media, esforco Baixo)
3. RF-042: Extract Interface (prioridade 0.65, severidade Baixa, esforco Baixo)

Claude: "Este codigo mostra caracteristicas de God Object (SMELL-03).

Prioridades de refactoring:

1. Extract Class (RF-018) - Prioridade 0.89
   - Severidade: Alta
   - Esforco: Medio
   - Mover metodos relacionados a pagamentos para a classe PaymentProcessor
   - Aplica: SRP (LAW-001), SoC (LAW-012)

2. Move Method (RF-023) - Prioridade 0.76
   - Severidade: Media
   - Esforco: Baixo
   - Realocar metodos de validacao para a classe Validator
   - Vitoria rapida para melhorar coesao

Comece com #2 (Move Method) para melhoria imediata, 
depois aborde #1 (Extract Class) para design de longo prazo."
```

---

## Agentes de IA

O Episteme inclui 4 agentes especializados que trabalham juntos como um sistema conectado. Cada agente usa ferramentas do grafo de conhecimento para analise e apresenta opcoes de acompanhamento interativas que podem transferir para outros agentes.

### Rede de agentes

```
code-reviewer ←→ episteme-advisor
      ↕                ↕
architecture-analyst ←→ episteme-researcher
      ↕
refactoring-expert (externo)
```

Cada agente encerra seu relatorio com **Proximos passos** — opcoes interativas que conectam a outros agentes ou ferramentas. Isso cria fluxos de trabalho de ponta a ponta, da deteccao a remediacao.

---

### 1. `code-reviewer`

**Quando usar**: Revisar codigo em busca de smells, violacoes SOLID e prioridades de refactoring

**O que faz diferente**: Vai alem da deteccao — rastreia cadeias de causalidade entre smells (quais sao causas-raiz vs. sintomas derivados), valida descobertas contra convencoes do ecossistema da linguagem (Rust, Go, etc.) e calibra a severidade contextualmente.

**Ferramentas usadas**: `analyze_code` → `suggest_refactorings` → `get_entity` → `get_neighbors` → `find_path`

**Exemplo de prompt**:
```
"Revise este modulo de autenticacao"
[colar codigo]
```

**Resposta do agente**:
```
# Revisao de codigo Episteme

## Resumo
AuthenticationModule tem um smell God Object como causa-raiz causando 3
problemas derivados. Corrigir a causa-raiz gerara melhorias em cascata.

## Deteccoes de smells
| Smell | Localizacao | Severidade | Confianca | Principio relacionado |
|-------|-------------|------------|-----------|----------------------|
| God Object (SMELL-03) | auth.rs:1-420 | Critica | confirmado | LAW-001 (SRP) |
| Long Method (SMELL-01) | auth.rs:88-142 | Alta | confirmado | LAW-001 (SRP) |
| Divergent Change (SMELL-09) | auth.rs:200-380 | Media | confirmado | LAW-012 (SoC) |
| Feature Envy (SMELL-15) | auth.rs:290-333 | Media | confirmado | LAW-003 (LSP) |

## Analise de causalidade
Causas-raiz:
- SMELL-03 → causa → SMELL-01, SMELL-09
  (corrigir God Object gerara melhorias em cascata)

Independente:
- SMELL-15 — independente, sem impacto derivado

## Refactorings classificados
1. **Extract Class** (RF-018) — Prioridade: Critica
   - Aborda: SMELL-03 (causa-raiz)
   - Efeito em cascata: tambem resolve SMELL-01, SMELL-09
   - O que fazer: Extrair classes SessionManager, EmailVerifier

## Proximos passos
1. Refatorar — Aplicar RF-018 (Extract Class) → usar agente refactoring-expert
2. Aprofundar — Explorar cadeia de causa-raiz para SMELL-03 → usar episteme-advisor
3. Verificacao arquitetural — Avaliar impacto estrutural → usar architecture-analyst
4. Saber mais — Obter explicacao completa de SMELL-03 → get_entity(detail_level="full")
```

**Validacao**: Antes de confirmar um smell, o agente verifica contra expressoes idiomaticas da linguagem (ex: structs grandes sao normais em Rust), contexto de dominio (arquivos de teste, codigo gerado) e impacto real (cosmetico vs. bloqueante para desenvolvimento).

---

### 2. `episteme-advisor`

**Quando usar**: Tomar decisoes de engenharia (selecao de padroes, compensacoes arquiteturais, aprofundamento de causa-raiz)

**Ferramentas usadas**: `search_knowledge` → `get_entity` → `get_neighbors` → `find_path`

**Exemplo de prompt**:
```
"Devo usar Singleton para minha classe ConfigManager?"
```

**Resposta do agente**:
```
# Assessoria: Singleton vs DI para ConfigManager

## Entidades aplicaveis
### 1. Singleton Pattern (DP-006) - Primario
- **O que diz**: Garantir que uma classe tenha apenas uma instancia
- **O que significa para voce**: Acesso global a config mas acoplamento oculto
- **Recomendacao**: Evitar — viola a testabilidade

### 2. Dependency Injection (DP-018) - Alternativa
- **O que diz**: Injetar dependencias via construtor
- **O que significa para voce**: Dependencias explicitas, simulacao facil
- **Recomendacao**: Preferir para ConfigManager

## Compensacoes
- Singleton (DP-006) oferece conveniencia mas viola DI (DP-018)
- DI adiciona codigo repetitivo mas habilita conformidade com LAW-005 (DIP)
- Resolucao: Usar DI com uma interface de provedor de configuracao

## Plano de acao
1. Imediato: Injetar ConfigManager via construtor
2. Curto prazo: Extrair interface ConfigProvider
3. Longo prazo: Considerar estrategias de configuracao especificas por ambiente

## Proximos passos
1. Implementar — Aplicar padrao DI → usar code-reviewer para validar
2. Explorar alternativas — Comparar abordagens → usar episteme-researcher
3. Impacto arquitetural — Avaliar efeitos derivados → usar architecture-analyst
```

---

### 3. `episteme-researcher`

**Quando usar**: Explorar relacoes no grafo de conhecimento, encontrar alternativas

**Ferramentas usadas**: `search_knowledge` → `get_entity` → `get_neighbors` → `find_path`

**Exemplo de prompt**:
```
"Quais leis o Padrao Observer aplica?"
```

**Resposta do agente**:
```
# Relatorio de pesquisa: Leis aplicadas pelo Padrao Observer

## Entidades primarias
1. **Observer Pattern** (DP-012) - Comportamental
   - Definicao: Definir dependencia um-para-muitos entre objetos
   - Quando a mudanca em um objeto altera outros automaticamente

## Mapa de conexoes
DP-012 --enforces--> LAW-002 (Open/Closed)
DP-012 --enforces--> LAW-005 (Dependency Inversion)
DP-012 --enforces--> LAW-001 (Single Responsibility)

## Contradicoes e compensacoes
- Observer (DP-012) habilita acoplamento fraco mas pode causar cadeias
  de atualizacao inesperadas — contrastado com Mediator (DP-014) que centraliza

## Proximos passos
1. Obter assessoria — Transformar descobertas em orientacao → usar episteme-advisor
2. Verificar codigo — Detectar Observer na sua base de codigo → usar code-reviewer
3. Avaliar arquitetura — Avaliar ajuste estrutural → usar architecture-analyst
```

---

### 4. `architecture-analyst`

**Quando usar**: Avaliar a arquitetura do sistema contra leis, padroes e riscos estruturais

**Ferramentas usadas**: `search_knowledge` → `get_entity` → `get_neighbors` → `find_path`

**Exemplo de prompt**:
```
"Esta arquitetura de microservicos segue a Lei de Conway?"
[descreve a arquitetura]
```

**Resposta do agente**:
```
# Analise arquitetural: Microservicos

## Cumprimento de leis e principios
| Principio | Estado | Evidencia | Impacto |
|-----------|--------|-----------|---------|
| Conway's Law (LAW-042) | violado | Shipping abrange 2 equipes | Sobrecarga de coordenacao |
| SRP (LAW-001) | em risco | Analytics depende de tudo | Acoplamento rigido |
| Bounded Context (LAW-031) | violado | Sem limites de dominio claros | Confusao de dados compartilhados |

## Tensoes-chave
- Conway's Law (LAW-042) requer alinhamento equipe↔servico
  mas Shipping service abrange as equipes Commerce + Platform
- Rastreado via: LAW-042 → related_to → LAW-001 → enforced_by → DP-026 (Strangler Fig)

## Recomendacoes arquiteturais
1. **Critica**: Mover Shipping para equipe Commerce — LAW-042 prevê falha de coordenacao
2. **Alta**: Introduzir Event Bus para Analytics — desacoplar via eventos assincronos
3. **Media**: Definir Bounded Contexts — alinhar limites de servico com o dominio

## Pontuacoes de conformidade
- Geral: 5/10 | Estrutura: 4/10 | Escalabilidade: 6/10 | Manutenibilidade: 5/10

## Proximos passos
1. Obter assessoria — Resolver tensoes-chave → usar episteme-advisor
2. Verificar codigo — Detectar smells estruturais → usar code-reviewer
3. Pesquisar alternativas — Encontrar melhores padroes → usar episteme-researcher
```

---

## Cadeias de fluxo de trabalho

Os agentes e ferramentas se conectam em pipelines de ponta a ponta. Cada cadeia produz um relatorio seguido de opcoes de acompanhamento interativas.

### Cadeia 1: Pipeline de revisao de codigo
```
analyze_code → suggest_refactorings → get_neighbors("solved_by")
  → find_path(smell_A, smell_B) → Relatorio com grafo de causalidade
  → Usuario escolhe: Aplicar correcao / Aprofundar / Verificacao arquitetural / Saber mais
```

### Cadeia 2: Pipeline de revisao arquitetural
```
search_knowledge → get_entity → get_neighbors("enforces")
  → get_neighbors("violates") → find_path → Relatorio de conformidade
  → Usuario escolhe: Plano de refactoring / Assessoria / Pesquisar alternativas
```

### Cadeia 3: Pipeline de diagnostico de problemas
```
search_knowledge(symptoms) → get_entity → get_neighbors("solved_by")
  → Relatorio de causa-raiz → Usuario escolhe: Aplicar correcao / Assessoria / Verificar
```

### Cadeia 4: Pipeline de aprendizado
```
search_knowledge(topic) → get_entity → get_neighbors("related_to")
  → Mapa de conceitos → Usuario escolhe: Exemplos de codigo / Aplicar ao codigo / Comparar
```

### Regras de encadeamento entre ferramentas

Cada chamada de ferramenta leva naturalmente a proxima:

| Apos chamar... | Sempre fazer acompanhamento com... |
|-----------------|-----------------------------------|
| `analyze_code` | `suggest_refactorings` nos smells detectados |
| `suggest_refactorings` | `get_neighbors(smell_id, "solved_by")` para alternativas |
| `search_knowledge` | `get_entity` nos 1-2 resultados principais |
| `get_entity` (smell) | `get_neighbors(id, "violates")` para principios impactados |
| `get_entity` (padrao) | `get_neighbors(id, "enforces")` para leis aplicadas |
| Multiplos smells detectados | `find_path(smell_A, smell_B)` para mapeamento de causalidade |

---

## Instalacao para outras ferramentas

### Cursor

```bash
epis install cursor
```

Adiciona configuracao MCP em `~/.cursor/mcp.json`:
```json
{
  "mcpServers": {
    "episteme": {
      "command": "episteme",
      "args": ["mcp"]
    }
  }
}
```

### Codex (OpenAI)

```bash
epis install codex
```

Gera `AGENTS.md` na raiz do projeto com definicoes de agentes.

### Integracao MCP personalizada

Se sua ferramenta suporta MCP, configure manualmente:

```json
{
  "mcpServers": {
    "episteme": {
      "command": "/path/to/episteme",
      "args": ["mcp"],
      "env": {
        "EPISTEME_DATA_DIR": "~/.episteme/data",
        "EPISTEME_DB_PATH": "~/.episteme/db/episteme.db"
      }
    }
  }
}
```

---

## Execucao como servico em segundo plano

Para melhor desempenho, execute o Episteme MCP como um proxy HTTP persistente:

```bash
# Iniciar servico em segundo plano
epis service start

# Verificar estado
epis service status
# Saida: Running on http://localhost:43175 (PID 12345)

# Habilitar inicio automatico na inicializacao (macOS)
epis service enable

# Parar servico
epis service stop
```

Atualize a configuracao MCP para usar o proxy HTTP:

```json
{
  "mcpServers": {
    "episteme": {
      "command": "episteme",
      "args": ["mcp", "--proxy", "http://localhost:43175"]
    }
  }
}
```

Registros: `~/.episteme/logs/mcp.out.log`

---

## Solucao de problemas

### Ferramentas nao aparecem no Claude

1. Verifique se o arquivo de configuracao existe: `cat ~/.claude/claude_desktop_config.json`
2. Verifique se episteme esta no PATH: `which episteme`
3. Teste MCP diretamente: `episteme mcp`
4. Verifique os registros: `tail -f ~/.episteme/logs/mcp.err.log`

### Erro "Database not found"

```bash
# Reconstruir a base de dados de conhecimento
epis build --rebuild
```

### Respostas de busca lentas

```bash
# Usar aceleracao GPU
epis build --gpu

# Ou executar como servico em segundo plano (aquecimento mais rapido)
epis service start
```

### Agente nao usa as ferramentas

Certifique-se de que o agente tenha capacidade de chamada de ferramentas. No Claude Code:
```
Usuario: "Use Episteme para encontrar padroes para logica de retry"
      ^^^^ mencione explicitamente o uso de ferramentas
```

---

## Avancado: Integracao de conhecimento personalizado

Combine Episteme (conhecimento generico) com Alcove (conhecimento da equipe):

```json
{
  "mcpServers": {
    "episteme": {
      "command": "episteme",
      "args": ["mcp"]
    },
    "alcove": {
      "command": "npx",
      "args": ["-y", "@joshuarileydev/alcove-mcp"]
    }
  }
}
```

Consulte o [Guia de integracao Alcove](./alcove-integration.md) para padroes de fonte dupla.

---

## Alternativa API

Se sua ferramenta de IA nao suporta MCP, use a API REST:

```bash
# Iniciar servidor API
docker-compose up -d

# Usar de qualquer ferramenta
curl http://localhost:8000/search?q=strategy+pattern
```

Consulte a [Documentacao da API](./api.md) para os endpoints.

---

## Ativacao automatica (Claude Code)

Quando voce descreve um problema em linguagem natural, o Claude Code detecta automaticamente a intencao e chama a ferramenta Episteme apropriada — **voce nao precisa mencionar o Episteme explicitamente**. Abaixo estao os padroes de ativacao exatos e exemplos.

### Como funciona

```
Sua entrada em linguagem natural
    ↓ Claude detecta palavras-chave/padroes
    ↓ A ferramenta Episteme e chamada automaticamente
    ↓ O grafo de conhecimento retorna dados verificados
    ↓ (Padroes de Design · Code Smells · Tecnicas de Refactoring · Leis de Engenharia)
    ↓ A resposta do Claude e fundamentada em evidencia
```

> **Nota:** Esta e uma ativacao automatica baseada em prompts, nao um gancho rigido. Para garantir uma chamada, use o skill `/episteme` diretamente.

### Problemas de estrutura de codigo

| O que voce diz (exemplos) | O que o Episteme detecta | Chamada automatica de ferramenta |
|--------------------------|-------------------------|--------------------------------|
| "Esta classe faz muito", "Este arquivo tem mais de 300 linhas" | God Class, Large Class, Single Responsibility | `search_knowledge("god class large class single responsibility")` |
| "Esta funcao e muito longa", "Linhas demais neste metodo" | Long Method | `search_knowledge("long method extract method")` |
| "O codigo e muito complexo", "Dificil de seguir" | Complexity, Cognitive Overload | `search_knowledge("complexity smell cognitive overload")` |
| "Copiei e colei isso em todo lugar", "Ha logica duplicada" | Duplicated Code, Clone | `search_knowledge("duplicated code clone smell")` |

### Problemas de acoplamento e dependencias

| O que voce diz (exemplos) | O que o Episteme detecta | Chamada automatica de ferramenta |
|--------------------------|-------------------------|--------------------------------|
| "Logica de negocio chama o DB diretamente" | Coupling, Persistence, Repository | `search_knowledge("coupling persistence repository data access layer")` |
| "Mudar X quebra Y", "Mudancas se propagam por todo lado" | Brittle Coupling, Change Propagation | `search_knowledge("brittle coupling change propagation rigidity")` |
| "Adicionar um novo tipo significa mexer em tudo", "switch-case so cresce" | Open/Closed, Strategy, Polymorphism | `search_knowledge("open closed principle strategy polymorphism")` |

### Problemas de teste e qualidade

| O que voce diz (exemplos) | O que o Episteme detecta | Chamada automatica de ferramenta |
|--------------------------|-------------------------|--------------------------------|
| "Isso e dificil de testar", "Nao consigo escrever testes unitarios para isso" | Testability, Dependency Injection | `search_knowledge("testability dependency injection mockability")` |

### Problemas de desempenho e concorrencia

| O que voce diz (exemplos) | O que o Episteme detecta | Chamada automatica de ferramenta |
|--------------------------|-------------------------|--------------------------------|
| "A API esta lenta", "O tempo de resposta esta muito alto" | N+1 Query, Lazy Loading, Caching | `search_knowledge("N+1 query lazy loading caching performance")` |
| "Isso e thread-safe?", "Ha problemas de concorrencia?" | Thread Safety, Race Condition | `search_knowledge("thread safety race condition shared mutable state")` |

### Revisao e analise de codigo

| O que voce diz (exemplos) | Chamada automatica de ferramenta |
|--------------------------|--------------------------------|
| Compartilha um trecho de codigo e pede revisao | `analyze_code(code)` → `suggest_refactorings(code)` |
| "Melhore este codigo", "Refatore isso para mim" | `suggest_refactorings(code)` |

### Discussoes de arquitetura

| O que voce diz (exemplos) | Chamada automatica de ferramenta |
|--------------------------|--------------------------------|
| "Microservicos vs monolito", "Como deveriamos dividir isso?" | `search_knowledge("monolith microservice decomposition bounded context")` |
| "Esta arquitetura esta boa?", "Revise este design" | `search_knowledge("layered architecture coupling responsibility")` |

### Exemplos de conversas do mundo real

**Exemplo 1 — Detectando um God Object:**

```
Voce: "UserService esta saindo do controle. Ele gerencia autenticacao,
        gerenciamento de sessoes, verificacao de email e redefinicao de senhas."

Claude (auto-chama search_knowledge):
  → Detecta: God Object / violacao de Single Responsibility
  → Retorna: SMELL-03 (God Object), LAW-001 (SRP), RF-018 (Extract Class)

Claude: "UserService mostra caracteristicas de God Object (SMELL-03).
  Viola o Principio da Responsabilidade Unica (LAW-001).

  Refactorings recomendados:
  1. Extract Class (RF-018) — Criar SessionManager para logica de sessoes
  2. Move Method (RF-023) — Mover verificacao de email para EmailService
  3. Extract Interface (RF-042) — Definir interface AuthService

  Comece com #2 para vitorias rapidas, depois #1 para melhoria estrutural."
```

**Exemplo 2 — Investigacao de desempenho:**

```
Voce: "A API de listagem de produtos esta muito lenta. Carrega bem individualmente
        mas a pagina de lista demora mais de 5 segundos."

Claude (auto-chama search_knowledge):
  → Detecta: Problema N+1 Query
  → Retorna: Padroes de desempenho, Lazy Loading, estrategias de Caching

Claude: "Isso soa como um problema N+1 query — cada produto aciona
  chamadas DB individuais em vez de uma consulta em lote.

  Solucoes do grafo de conhecimento:
  1. Batch loading / Eager loading — Obter dados relacionados em uma consulta
  2. Data Mapper (DP-019) — Desacoplar logica de consulta de objetos de dominio
  3. Aplicar Lazy Loading (RF-050) — Carregar relacoes apenas sob demanda

  Gostaria que eu verificasse o codigo para padroes N+1?"
```

**Exemplo 3 — Revisao de codigo com trecho:**

```
Voce: [colar codigo Python]
       "Voce pode revisar isso? Algo nao parece certo."

Claude (auto-chama analyze_code → suggest_refactorings):
  → analyze_code detecta: Long Method (confianca 0.87), Long Parameter List (0.92)
  → suggest_refactorings retorna sugestoes classificadas

Claude: "Encontrei 2 code smells:

  1. Long Method (SMELL-01) — 45 linhas, complexidade ciclomatica 12
     → Extract Method (RF-001), prioridade 0.89

  2. Long Parameter List (SMELL-08) — 9 parametros
     → Introduce Parameter Object (RF-029), prioridade 0.92

  Comece com RF-029 (Parameter Object) — e a prioridade mais alta
  e facilita o Extract Method subsequente."
```

---

## Proximos passos

1. **Experimentar agentes**: Pergunte ao episteme-advisor "Devo usar Singleton?"
2. **Analisar codigo**: Cole uma funcao e peca ao code-reviewer para verificar smells
3. **Explorar o grafo**: Use episteme-researcher para encontrar relacoes entre padroes
4. **Fluxos de trabalho personalizados**: Combine ferramentas (analyze → suggest → search)

Para mais exemplos, consulte:
- [Integracao Alcove](./alcove-integration.md) — Conhecimento da equipe + Episteme
- [Configuracao de monitoramento](../../monitoring/README.md) — Rastrear uso de padroes
- [Referencia API](./api.md) — Endpoints REST
