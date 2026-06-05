<p align="center">
<img src="../assets/icon.png" alt="Episteme" width="60%" />
</p>

<p align="center"><sub>Episteme (συν ταγμα) — do grego para "sistema organizado" ou "discernimento"</sub></p>

<p align="center">Um grafo de conhecimento offline-first, de binário único, que conecta padrões de projeto, técnicas de refatoração e leis de software através de relações semânticas.<br><b>Construído primeiro para agentes de IA</b> — integre expertise em engenharia de software diretamente no Claude Code, Cursor e outras ferramentas compatíveis com MCP.</p>

<p align="center">Escrito em Rust · Binário único · Totalmente offline</p>

---

<p align="center">
    <a href="https://github.com/epicsagas/Episteme/actions"><img src="https://img.shields.io/github/actions/workflow/status/epicsagas/Episteme/ci.yml?branch=main&label=CI" alt="CI" /></a>
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.95+-orange.svg" alt="rust-lang" /></a>
    <a href="https://crates.io/crates/episteme"><img src="https://img.shields.io/badge/crates.io-v0.1.0-orange.svg" alt="crates.io" /></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License" /></a>
    <a href="https://buymeacoffee.com/epicsaga"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-FFDD00?style=flat&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me a Coffee" /></a>
</p>

<p align="center">
  <a href="../../README.md">English</a> |
  <a href="../ja/">日本語</a> |
  <a href="../ko/">한국어</a> |
  <a href="../de/">Deutsch</a> |
  <a href="../fr/">Français</a> |
  <a href="../zh-CN/">简体中文</a> |
  <a href="../zh-TW/">繁體中文</a> |
  Português |
  <a href="../es/">Español</a> |
  <a href="../hi/">हिन्दी</a>
</p>

---

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/features.png">
  <img src="../assets/features.png" align="center" width="100%" alt="Visão Geral das Funcionalidades do Episteme" />
</picture>

---

## Início Rápido

### Claude Code

```
/plugin marketplace add epicsagas/plugins
/plugin install episteme@epicsagas
```

O hook do plugin instala o binário `epis` automaticamente. **Antes de iniciar uma nova sessão**, execute este comando uma vez no seu terminal:

```bash
epis install   # Baixa os dados do grafo de conhecimento do GitHub Releases
```

`epis install` inicializa o banco de dados do grafo de conhecimento e inicia o servidor HTTP API na porta 58302. Em seguida, inicie uma nova sessão do Claude Code e está pronto.

Atualizar: `/plugin update episteme@epicsagas`

### Codex CLI

```bash
codex plugin marketplace add epicsagas/plugins
```

O hook do plugin instala o binário `epis` automaticamente. **Antes de iniciar uma nova sessão**, execute este comando uma vez no seu terminal:

```bash
epis install   # Baixa os dados do grafo de conhecimento do GitHub Releases
```

`epis install` inicializa o banco de dados do grafo de conhecimento e inicia o servidor HTTP API na porta 58302. Em seguida, inicie uma nova sessão e está disponível imediatamente.

Atualizar: `codex plugin update episteme@epicsagas`

### Outras ferramentas

```bash
epis install cursor       # Cursor IDE
epis install opencode     # OpenCode
epis install cline        # Cline
epis install --all        # Todas as ferramentas suportadas
```

### Instalação manual

| Método | Comando |
|--------|---------|
| **Homebrew** | `brew install epicsagas/tap/episteme` |
| **Script shell** | `curl --proto '=https' --tlsv1.2 -LsSf https://github.com/epicsagas/Episteme/releases/latest/download/episteme-installer.sh \| sh` |
| **PowerShell** | `irm https://github.com/epicsagas/Episteme/releases/latest/download/episteme-installer.ps1 \| iex` |
| **cargo** | `cargo binstall episteme` ⚡ ou `cargo install episteme` |
| **Docker** | Veja [Opção 3](#opção-3-docker-sem-rust-necessário) |

### Verificar

```bash
epis --version
epis stats
```

Ou de dentro do Claude Code / Codex CLI:

```
/episteme verify
```

### Experimente em 30 segundos

**Opção A — CLI:** Aponte para qualquer arquivo no seu projeto.

```bash
epis analyze src/domain/engine.rs
```

```
✓ 2 smells detectados em src/domain/engine.rs

  SMELL-07 (Large Class) — RefactoringRanker, 743 linhas
  → RF-018 Extract Class          prioridade 0.89  esforço: médio
  → RF-001 Extract Method         prioridade 0.76  esforço: pequeno
  → Viola: LAW-001 Single Responsibility Principle

  SMELL-01 (Long Method) — rank_refactorings(), 58 linhas
  → RF-001 Extract Method         prioridade 0.92  esforço: pequeno
  → Viola: LAW-001 SRP, LAW-004 DRY
```

**Opção B — Claude Code:** Abra qualquer arquivo no seu projeto e pergunte naturalmente.

```
Encontre code smells neste projeto e sugira refatorações.
```

O Episteme é acionado automaticamente — sem necessidade de sintaxe especial. Ele mapeia sua descrição para o grafo de conhecimento e retorna resultados classificados e citáveis.

---

## Por que Episteme?

LLMs já sabem o que é o padrão Strategy. Eles podem recitar os princípios SOLID, listar os padrões GoF e explicar code smells. Então por que este projeto existe?

**A lacuna não é conhecimento — é raciocínio estruturado e conectado.**

Quando você pergunta a um LLM "como eu corrijo um God Object?", ele dá uma resposta razoável. Mas a resposta muda entre conversas, falta rastreabilidade e não conecta o problema às suas causas raiz ou consequências posteriores. O Episteme transforma fatos isolados em um grafo navegável onde cada recomendação é fundamentada, citável e conectada ao panorama mais amplo do design.

### Como isso é diferente de apenas usar prompts bem elaborados em um LLM?

| | Prompt bem elaborado no LLM | Episteme + LLM |
|---|---|---|
| Detecção proativa | Somente se o usuário fizer a pergunta certa | Aciona automaticamente a partir de descrições de problemas |
| Eficiência de tokens | Explicações longas + múltiplas rodadas de acompanhamento | Uma chamada de ferramenta retorna resultado estruturado |
| Travessia de relações | Um salto no máximo, frequentemente alucinado | Travessia de grafos multi-salto, verificada |
| Referência cruzada | Manual, propensa a erros | Automatizada via 201 relações semânticas |
| Consistência | Varia entre conversas | Mesma resposta estruturada todas as vezes |
| Citabilidade | "Eu acho que você deveria usar Extract Class" | "Extract Class (RF-018), prioridade 0.89" |
| Offline / Ambiente isolado | Requer internet para melhores resultados | Totalmente local, binário único |

### Quando isso é útil?

<details>
<summary><b>1. Quando seu agente de IA deve detectar problemas proativamente, não esperar para ser questionado</b></summary>

A integração MCP é acionada automaticamente a partir de descrições de problemas. Quando um usuário diz "esta classe faz coisas demais", o agente não precisa saber perguntar sobre God Object — o Episteme mapeia a reclamação para `SMELL-03`, apresenta refatorações classificadas e rastreia a violação até os primeiros princípios. Isso transforma uma reclamação vaga em um plano de correção estruturado.
</details>

<details>
<summary><b>2. Quando você quer reduzir o consumo de tokens — não gastá-los com explicações</b></summary>

Sem o Episteme, um LLM responde "como eu corrijo um God Object?" explicando o smell, listando refatorações, descrevendo princípios SOLID e percorrendo cada opção — centenas de tokens por resposta. Com o Episteme, uma chamada de ferramenta MCP retorna `SMELL-03 → RF-018 (0.89) → LAW-001`. A mesma expertise com uma fração do orçamento de tokens.
</details>

<details>
<summary><b>3. Quando você precisa de análise de código conectada à correção — não apenas detecção</b></summary>

Ferramentas como SonarQube detectam smells. LLMs podem sugerir padrões. O Episteme faz ambos e os conecta: detecta Long Method → rastreia as leis que ele viola → classifica as refatorações que o resolvem → mostra quais padrões reforçam essas refatorações.
</details>

<details>
<summary><b>4. Quando o conhecimento isolado de padrões não é suficiente — você precisa das relações</b></summary>

Saber o que Extract Method faz é o básico. Saber que ele *resolve* Long Method (SMELL-01), que *viola* Single Responsibility (LAW-001), que é *reforçado pelo* Facade Pattern (DP-012) — essa é uma cadeia de raciocínio que um LLM não consegue construir de forma confiável por conta própria. As 201 relações semânticas do Episteme permitem que agentes de IA percorram esses caminhos de forma determinística.
</details>

<details>
<summary><b>5. Quando você está tomando decisões de arquitetura e precisa de evidências, não opiniões</b></summary>

"Devo usar microsserviços?" — O Episteme conecta a pergunta à Lei de Conway (LAW-017), SRP (LAW-001) e ao padrão Strangler Fig (DP-026), mostrando como eles se relacionam. As decisões tornam-se rastreáveis até leis de engenharia, não posts de blog.
</details>

<details>
<summary><b>6. Quando você precisa de conselhos de engenharia consistentes e citáveis — não recomendações alucinadas</b></summary>

Cada descoberta referencia IDs de entidades explícitos (`DP-005`, `RF-001`, `LAW-021`). As recomendações vêm com pontuações de prioridade e estimativas de esforço. A mesma consulta sempre retorna a mesma resposta estruturada.
</details>

<details>
<summary><b>7. Quando você está trabalhando em um ambiente isolado ou rede restrita</b></summary>

O Episteme funciona inteiramente offline: binário único, banco de dados SQLite local, embeddings locais via fastembed (ONNX Runtime). Sem telemetria, sem chamadas para servidores externos, sem APIs externas. Seu código e resultados de análise nunca saem da sua máquina.
</details>

---

## Funcionalidades

| | Funcionalidade | Por que é importante |
|--|----------------|----------------------|
| 🧠 | **22 Padrões de Projeto GoF** | Catálogo completo com exemplos reais |
| 🔧 | **66 Técnicas de Refatoração** | Catálogo de Fowler com exemplos de código |
| ⚖️ | **56 Leis e Princípios de Software** | SOLID, Lei de Conway, Teorema CAP, etc. |
| 👃 | **17 Tipos de Code Smells** | Long Method, God Object, Feature Envy, etc. ¹ |
| 🔗 | **201 Relações Semânticas** | "resolve", "impõe", "viola", "relaciona-se com" |
| 🤖 | **9 Ferramentas MCP + 4 Agentes** | Interação de agente IA de alta fidelidade com transferências entre agentes |
| 🌐 | **Servidor HTTP API** | API REST na porta 58302, iniciado automaticamente na instalação |
| 🌍 | **Suporte a 10 Linguagens** | Python (AST), Java, TypeScript, Go, Rust, C++, C#, PHP, Ruby, Kotlin |
| 📊 | **Análise Determinística** | Python baseado em AST + regex multilinguagem, mesmo resultado sempre |
| 🏷️ | **Conhecimento Citável** | Cada descoberta é vinculada a IDs de entidade explícitos (`RF-001`, `LAW-021`) |
| 🌐 | **API REST (17 endpoints)** | Autenticação, limite de taxa, probes de saúde, métricas Prometheus |
| 📦 | **Binário Único** | Sem runtime, multiplataforma (macOS, Linux, Windows) |
| 🔌 | **Embeddings Locais** | fastembed (ONNX Runtime), busca semântica sem configuração |
| 🐳 | **Suporte Docker** | Build multi-estágio com verificações de saúde |

> ¹ Duplicate Code (SMELL-13) e Shotgun Surgery (SMELL-09) requerem contexto de múltiplos arquivos e são ignorados no modo de arquivo único.

---

## Instalação

### Opção 1: cargo-binstall (Recomendado)

```bash
cargo binstall episteme    # baixa binário pré-compilado — sem compilação necessária
epis install cursor        # popula dados + inicia servidor API + instala agentes
```

Se não tiver cargo-binstall: `cargo install cargo-binstall`

> Após `epis install`, o servidor HTTP API inicia automaticamente na porta 58302. MCP continua disponível -- veja `registry/mcp.json` para configuração manual.

### Opção 2: A Partir do Código-Fonte

```bash
git clone https://github.com/epicsagas/Episteme.git
cd Episteme && cargo build --release
```

Depois execute o binário para sua plataforma:

| Plataforma | Comando |
|------------|---------|
| **macOS / Linux** | `./target/release/epis install --local cursor` |
| **Windows** | `.\target\release\episteme.exe install --local cursor` |

### Opção 3: Docker (Sem Rust Necessário)

```bash
docker-compose up -d
```

Adicione ao seu arquivo de configuração MCP:

| Ferramenta | Caminho do arquivo de configuração |
|------------|-------------------------------------|
| Claude Code | `~/.claude.json` |
| Cursor | `.cursor/mcp.json` |
| VS Code (Copilot) | `.vscode/mcp.json` |

```json
{
  "mcpServers": {
    "episteme": {
      "command": "docker",
      "args": ["exec", "-i", "episteme-api", "episteme", "mcp"]
    }
  }
}
```

### Opção 4: Binários Pré-compilados (Sem Rust Necessário)

Baixe o binário mais recente para sua plataforma em [GitHub Releases](https://github.com/epicsagas/Episteme/releases):

| Plataforma | Arquivo |
|----------|------|
| **macOS** (Apple Silicon) | `episteme-aarch64-apple-darwin.tar.xz` |
| **Linux** (x86_64) | `episteme-x86_64-unknown-linux-gnu.tar.xz` |
| **Linux** (ARM64) | `episteme-aarch64-unknown-linux-gnu.tar.xz` |
| **Windows** (x86_64) | `episteme-x86_64-pc-windows-msvc.zip` |

```bash
# macOS / Linux
tar xzf episteme-*.tar.gz
sudo mv episteme /usr/local/bin/

# Windows — extraia o zip e adicione episteme.exe ao seu PATH
```

Depois instale:
```bash
epis install cursor
```

### Verificar

```bash
epis --version
epis stats
epis explore "strategy pattern"    # explore o grafo de conhecimento
```

Ou de dentro do Claude Code / Codex CLI:

```
/episteme verify
```

---

## Endpoints HTTP API

> O Episteme funciona como um servidor HTTP API sempre ativo na porta 58302. Skills e agentes usam `curl http://localhost:58302/...` em vez de ferramentas MCP. MCP continua disponível para configuração manual -- veja `registry/mcp.json`.

### Endpoints da API

#### Grafo de Conhecimento

| Método | Endpoint | Propósito |
|--------|----------|-----------|
| **GET** | `/health` | Verificação de saúde |
| **GET** | `/search?q=...` | Pesquisar no grafo de conhecimento |
| **GET** | `/graph/{id}` | Obter entidade por ID |
| **GET** | `/graph/{id}/neighbors` | Obter entidades relacionadas |
| **POST** | `/graph/path` | Encontrar caminho entre duas entidades |

#### Análise de Código

| Método | Endpoint | Propósito |
|--------|----------|-----------|
| **POST** | `/analyze` | Detectar code smells |
| **POST** | `/refactor` | Sugerir refatorações |

#### Conhecimento Tácito

| Método | Endpoint | Propósito |
|--------|----------|-----------|
| **POST** | `/insights` | Adicionar insight da equipe |

### 9 Ferramentas MCP (Legacy)

#### Conhecimento canonico (6 ferramentas)

| Ferramenta | Proposito | Exemplo de uso |
|------|---------|-------------|
| **`search_knowledge`** | Pesquisa semantica em todas as entidades | "Encontrar padroes para logica de retry" |
| **`get_entity`** | Obter detalhes de entidade especifica por ID | "Explicar Strategy Pattern (DP-023)" |
| **`get_neighbors`** | Explorar entidades relacionadas | "Quais refactorings resolvem Long Method?" |
| **`find_path`** | Encontrar conexao entre duas entidades | "Como SRP se relaciona com Extract Class?" |
| **`analyze_code`** | Detectar code smells via regex/AST | "Revisar este codigo de validacao de pagamento" |
| **`suggest_refactorings`** | Sugestoes de refatoracao classificadas | "O que devo refatorar nesta classe?" |

#### Conhecimento tacito (3 ferramentas)

| Ferramenta | Proposito | Exemplo de uso |
|------|---------|-------------|
| **`add_insight`** | Registar decisoes da equipa, licoes aprendidas | "Escolhemos event-driven em vez de polling por razao X" |
| **`search_insights`** | Pesquisar conhecimento passado da equipa | "O que decidimos sobre o middleware de autenticacao?" |
| **`confirm_links`** | Validar links detetados automaticamente para entidades canonicas | Confirmar que TK-001 se relaciona com SMELL-03 |

O Episteme armazena o conhecimento tacito numa base de dados separada (`~/.episteme/user_knowledge.db`) e funde-o com o grafo canonico em tempo de execucao atraves de uma camada composta. As perspetivas da equipa sao automaticamente ligadas a padroes, leis e smells, transformando a experiencia em conhecimento navegavel.

Consulte [Arquitetura do conhecimento tacito](./tacit-knowledge.md) para o design completo.

### 4 Agentes Especializados (Rede Conectada)

Os agentes trabalham juntos — cada análise termina com opções de **Próximos Passos** que encaminham para outros agentes.

| Agente | Quando Usar | Capacidade Principal | Encaminha para |
|--------|-------------|---------------------|----------------|
| **`code-reviewer`** | Code smells, violações SOLID | Análise de causalidade (causa raiz → sintomas posteriores) | advisor, architecture-analyst, refactoring-expert |
| **`episteme-advisor`** | Decisões de engenharia, trade-offs | Cadeias de trade-offs multi-entidade com planos de ação | code-reviewer, architecture-analyst, researcher |
| **`episteme-researcher`** | Exploração do grafo de conhecimento | Mapas de conexão entre padrões, leis, smells | advisor, code-reviewer |
| **`architecture-analyst`** | Avaliação de arquitetura contra leis | Pontuação de conformidade com avaliação ponderada por risco | advisor, code-reviewer, researcher |

**Exemplo de fluxo de trabalho**: `code-reviewer` detecta God Object → rastreia causalidade para 3 smells posteriores → oferece "Aplicar RF-018" (→ refactoring-expert) ou "Aprofundar causa raiz" (→ episteme-advisor) ou "Verificação de arquitetura" (→ architecture-analyst).

[Guia Completo de Integração MCP](./mcp-integration-guide.md)

---

## Uso via CLI

```bash
# Analisar código em busca de smells
epis analyze my_code.py --language python --json
episteme infer my_code.py

# Explorar o grafo de conhecimento
epis explore "strategy pattern"
epis graph path DP-005 RF-001   # ex: Factory Method → Extract Method

# Construir o índice RAG
epis build

# Iniciar servidores
epis api              # REST API na porta :58302
episteme mcp --http       # Servidor MCP na porta :43175 (legacy)
episteme web --port 8080  # Interface Web (explorador de grafo interativo)

# Empacotamento para distribuição
episteme dist --out-dir release/
```

---

## Documentação

| Documento | Descrição |
|-----------|-----------|
| [Início Rápido](./QUICKSTART.md) | Configuração passo a passo, primeira execução, solução de problemas |
| [Guia de Integração MCP](./mcp-integration-guide.md) | Referência de ferramentas, exemplos de agentes, fluxos de conversação |
| [Arquitetura do conhecimento tacito](./tacit-knowledge.md) | Projeto com dois bancos de dados, ciclo de vida de insights, esquema |
| [Comparacao do ecossistema Alcove](./alcove-ecosystem.md) | Modelos de armazenamento, capacidades de busca, matriz de casos de uso |
| [Guia de integracao do Alcove](./alcove-integration.md) | Fluxos de contexto duplo, configuracao, melhores praticas |
| [Referência da API](./api.md) | Endpoints REST, autenticação, exemplos |
| [Distribuição](./distribution.md) | Empacotamento de release e implantação |
| [Desenvolvimento e Contribuição](./DEVELOPMENT.md) | Arquitetura, como contribuir |
| [Registro de Alterações](./CHANGELOG.md) | Histórico de releases e notas de versão |

---

## Configuração

### Variáveis de Ambiente

```bash
# Locais de dados
EPISTEME_DATA_DIR=~/.episteme/data
EPISTEME_DB_PATH=~/.episteme/db/episteme.db

# Servidor API
EPISTEME_API_HOST=0.0.0.0
EPISTEME_API_PORT=58302
EPISTEME_API_KEY=your-secret-key

# Servidor MCP
EPISTEME_MCP_HOST=127.0.0.1
EPISTEME_MCP_PORT=43175
```

---

## Solução de Problemas

**Comando `episteme` não encontrado após a instalação**

| Plataforma | Solução |
|------------|---------|
| **macOS / Linux** | `export PATH="$HOME/.cargo/bin:$PATH"` — adicione ao `~/.bashrc` ou `~/.zshrc` para persistir |
| **Windows** | Adicione `%USERPROFILE%\.cargo\bin` ao PATH do sistema, ou abra um novo terminal |

**Ferramentas MCP não aparecendo no Claude Code / Cursor**

O servidor HTTP API inicia automaticamente na porta 58302 após `epis install`. Skills usam `curl http://localhost:58302/...` para interagir com o Episteme. MCP continua disponível para configuração manual -- veja `registry/mcp.json`.

**Porta já em uso**
```bash
epis api --port 58303   # use uma porta diferente
```

**Primeira inicialização lenta**

O Episteme constrói um índice de embeddings local na primeira execução. Isso leva de 30 a 60 segundos e é um custo único. Inicializações subsequentes são instantâneas.

**Erros de compilação durante `cargo install`**

Certifique-se de que o Rust 1.95+ esteja instalado:
```bash
rustup update stable
rustup show   # confirme a toolchain ativa
```

> Mais ajuda: [Seção de solução de problemas do QUICKSTART.md](../../QUICKSTART.md#troubleshooting) · [Abra uma issue](https://github.com/epicsagas/Episteme/issues)

---

## Roteiro

**Lançado**
- [x] `epis install` — configuração de dados com um único comando a partir do GitHub Releases
- [x] Homebrew tap (`epicsagas/tap/episteme`) — macOS Apple Silicon + Linux (x86_64 + ARM64)
- [x] Suporte ao marketplace de plugins do Claude Code & Codex CLI
- [x] Traduções do README — 9 idiomas (ko, ja, zh-CN, zh-TW, de, fr, es, pt, hi)
- [x] **Builds multiplataforma** — macOS, Linux, Windows (com aceleração GPU DirectML)

**Planejado**
- [ ] **Entidades Personalizadas** — Adicionar padrões/smells específicos da equipe
- [ ] **Metadados Multilíngues** — Títulos e resumos de entidades em idiomas CJK
- [ ] **Tutoriais Interativos** — Tours guiados no aplicativo para ferramentas MCP
- [ ] **Métricas de Equipe** — Agregação de uso de padrões pela organização

---

## Contribuindo

Contribuições são bem-vindas! Veja [DEVELOPMENT.md](./DEVELOPMENT.md) para a visão geral da arquitetura e guia de contribuição.

```bash
# Executar testes
cargo test

# Lint
cargo clippy -- -D warnings

# Formatação
cargo fmt
```

Dúvidas? [Abra uma discussão](https://github.com/epicsagas/Episteme/discussions) ou [registre uma issue](https://github.com/epicsagas/Episteme/issues).

---

## Licença

Apache 2.0 — veja [LICENSE](../../LICENSE) para detalhes.
