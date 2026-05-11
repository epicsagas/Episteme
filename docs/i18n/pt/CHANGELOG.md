# Changelog

Todas as alteracoes notaveis no Episteme serao documentadas neste arquivo.

O formato e baseado em [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
e este projeto adere ao [Versionamento Semantico](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Alterado

- CLI: `explore` renomeado para `search` (nome antigo funciona como alias obsoleto)
- CLI: `mcp` e `api` agora gerenciam seu ciclo de vida completo de servico (`start`, `stop`, `restart`, `status`, `enable [--now]`, `disable [--now]`)
- CLI: comando de nivel superior `service` obsoleto — use `mcp start/stop/restart/status/enable/disable` em vez disso
- CLI: `mcp --http` obsoleto — use `mcp start` para modo daemon HTTP
- CLI: `launchd-install/uninstall/status` obsoleto — use `mcp enable/disable/status` em vez disso
- `enable/disable` agora multiplataforma: macOS (launchd) e Linux (unidade de usuario systemd)

### Adicionado

- `api start/stop/restart/status/enable/disable` — Gerenciamento do ciclo de vida do daemon da API REST
- Geracao de unidade de usuario systemd Linux para `mcp enable`

- **Transporte HTTP MCP para Claude Code** — seletor de transporte TUI, HTTP como padrao, auto-enable launchd
- **Auto-instalacao de prompts de agente** — `epis install` copia prompts de agente Episteme para `~/.claude/agents/`
- **Descricoes de entidades** — campo de descricao extraido automaticamente dos arquivos markdown de origem, exibido no painel de detalhes do visualizador web
- **SPA de visualizacao de benchmarks** — analise de tendencias, painel de detalhamento de consultas
- **Redesign do visualizador web** — layout de diagrama Sankey, arvore na barra lateral, painel de detalhes, melhorias de legibilidade do subgrafo
- **Upsert de configuracao MCP** — re-executar `epis install` atualiza o transporte quando a configuracao difere (stdio ↔ HTTP)
- **Configuracao MCP yaml** — `mcp.host` / `mcp.port` em `config.yaml` (yaml → fallback env)
- **Monitoramento** — suporte nativo e remoto de destino de coleta Prometheus via variaveis de ambiente
- **Endurecimento de CI** — cargo audit, gitleaks, geracao de SBOM, SHAs de acoes fixados
- **Pipeline de release** — target Windows, publicacao no crates.io, tap Homebrew
- **Exemplo de diagnostico arquitetonico de God module** em `examples/`

### Alterado

- **Assistente de instalacao** — todas as etapas (transporte, Redis, telemetria) migradas para TUI de tela cheia
- **Fluxo de instalacao** — constroi indice RAG automaticamente apos preparacao, ignora quando DB ja existe
- **Grafo de conhecimento** — enriquecido com relacoes semanticas entre entidades
- **Licenca** — MIT → Apache-2.0

### Corrigido

- Panic do runtime Tokio no `main()` sincrono para telemetria
- Qualidade de busca — bug de medicao NDCG resolvido, precisao hit@1 melhorada para 100%
- Recall de busca — boost entre tipos, tratamento de entidades esparsas, sinonimos de intencao
- Cache do modelo fastembed fixado em `~/.episteme/models`
- Substituicao de UID bootstrap do launchd e tratamento de porta em uso
- Origens CORS agora configuraveis via `EPISTEME_CORS_ORIGINS`

## [0.1.0] - 2026-05-03

### Adicionado

- **Reescrita completa em Rust** — substituicao total da base de codigo Python por Rust idiomativo
- **Arquitetura hexagonal** — `ports/` (traits), `domain/` (logica de negocio), `adapters/` (infraestrutura), `server/` (HTTP)
- **Framework GenericParser** — 8 parsers baseados em chaves consolidados em `GenericParser` com `ParserConfig`; padroes regex armazenados em cache via `OnceLock` com `Box::leak`
- **Parsing AST Python** — `rustpython-parser` para deteccao precisa de smells Python (Long Method, Large Class, God Object)
- **TieredAccum + build_detection()** — desduplicou 14 construcoes identicas de deteccao de smells em `detectors.rs` (1.253 → 591 linhas)
- **Decomposicao do modulo MCP** — dividiu `EpistemeMCP` (675 linhas) em servicos `mcp_search`, `mcp_graph`, `mcp_analysis`
- **Decomposicao de comandos CLI** — dividiu `main.rs` (1.741 linhas) em modulo `commands/` com `cli.rs` para definicoes clap
- **Desduplicacao de handlers API** — mesclou `search`/`search_post` duplicados em `do_search()` compartilhado
- **16 funcoes detectoras de smells** — aumento de 14, cobrindo todas as categorias de smells do GoF
- **17 endpoints da API REST** — sondas de saude, metricas Prometheus, CORS, limitacao de taxa
- **Eviccao TTL do limitador de taxa** — MAX_BUCKETS=10.000 com TTL de 1 hora para evitar crescimento ilimitado de memoria
- **Mitigacao ReDoS** — regex de operador ternario limitado de `[^:]+` para `[^:\n]{1,50}`
- **Embeddings locais** — fastembed (ONNX Runtime) para busca semantica sem configuracao
- **Assistente de instalacao interativo** — TUI com crossterm, atalhos vim, tela alternativa
- **Empacotamento de distribuicao** — comando `episteme dist` para criacao de pacote de release com bootstrap automatico do DB
- **CI multiplataforma** — Workflow de release GitHub Actions para linux/macOS (x86_64 + aarch64)
- **Dockerfile multi-stage** — Builder Rust + runtime Debian slim

### Alterado

- **Linguagem**: Python 3.11+ → Rust (edicao 2024)
- **Framework web**: FastAPI → axum
- **Banco de dados**: Python sqlite3 → rusqlite (integrado)
- **Embeddings**: sentence-transformers/PyTorch → fastembed/ONNX Runtime
- **CLI**: argparse → clap (derive)
- **Todos os padroes regex em cache** — zero recompilacao em caminhos criticos via `REGEX_CACHE` global

### Removido

- Dependencia do runtime Python
- Dependencia ChromaDB
- Dependencia tree-sitter
- Workflow de publicacao PyPI
- Binario standalone `episteme-hook` (era entry point PyPI apenas Python) — use `episteme hooks ground|sniff|audit` em vez disso

## [0.0.5] - 2026-04-30

### Adicionado

- UI web de visualizacao do grafo (`episteme web`) com D3-force
- Base de vetores pre-construida no pacote de release
- Flag `epis install --local` para fluxos de trabalho de desenvolvimento
- 650+ relacoes semanticas cobrindo todas as 161 entidades
- CI gera automaticamente base de vetores durante release

## [0.0.4] - 2026-04-29

### Adicionado

- Servidor MCP com 6 ferramentas
- 4 agentes especializados
- Comando `epis install`
- Gerenciamento de daemon `epis service`
- Busca hibrida (FTS5 + vetorial)
- Cache Redis, aceleracao GPU
- Deteccao de code smells em 10 linguagens
- Monitoramento Prometheus + Grafana
