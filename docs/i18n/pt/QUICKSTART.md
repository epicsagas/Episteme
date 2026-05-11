# Episteme — Guia de inicio rapido

Comece a usar o Episteme em menos de 2 minutos.

---

## Pre-requisitos

- **Rust 1.95+** (edicao 2024 necessaria) — [Instalar via rustup](https://rustup.rs)
- Conexao com a internet (para download inicial dos dados)

---

## Opcao 1: Integracao com ferramenta de IA (Recomendado)

**Ideal para:** Usuarios de Claude Code, Cursor, Codex, Gemini

```bash
# 1. Instalar Episteme
cargo install --git https://github.com/epicsagas/Episteme

# 2. Instalar na sua ferramenta de IA (baixa dados, configura MCP, copia agentes)
epis install claude      # Claude Code
epis install cursor      # Cursor
epis install codex       # OpenAI Codex
epis install gemini      # Gemini CLI
epis install all         # Todas as ferramentas de uma vez
```

> Se `epis install claude` falhar ao baixar os dados, use a instalacao via codigo-fonte abaixo.

**Pronto.** Reinicie sua ferramenta de IA e o Episteme estara ativo.

---

## Opcao 2: Docker (Sem necessidade de Rust)

```bash
docker-compose up -d

# Acesso
# API:       http://localhost:8000
# Health:    http://localhost:8000/health
```

Para integracao MCP via Docker, adicione a sua configuracao MCP:
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

---

## Opcao 3: A partir do codigo-fonte

```bash
git clone https://github.com/epicsagas/Episteme.git
cd Episteme

# Compilar
cargo build --release

# Preparar dados e construir base de vetores (build executa automaticamente)
./target/release/epis install --local
```

---

## Visualizacao do grafo

O Episteme inclui um visualizador de grafo interativo com D3-force:

```bash
episteme web               # padrao: http://localhost:8080
episteme web --port 9001   # porta personalizada
episteme web --host 0.0.0.0 --port 8080  # expor para a rede
```

---

## Comandos comuns

```bash
# Analisar codigo em busca de smells
epis analyze my_code.py --language python
epis analyze my_code.py --json

# Obter sugestoes de refactoring
episteme infer my_code.py --top-k 5

# Explorar o grafo de conhecimento
epis explore "strategy pattern"
epis graph path DP-005 RF-001

# Iniciar servidores
epis api              # REST API na porta :8000
episteme mcp --http       # Servidor MCP na porta :43175
episteme web --port 8080  # Interface Web

# Daemon MCP em segundo plano (proxy HTTP)
epis service start
epis service status
epis service stop

# Criar pacote de release
episteme dist --out-dir release
```

---

## Solucao de problemas

### "Database not found"
```bash
epis install claude   # baixar novamente o pacote de dados
# ou
epis install --local
```

### "Port already in use"
```bash
episteme web --port 9001
epis api --port 9000
```

---

## Proximos passos

- **[README](../../README.md)** — Visao geral completa de recursos e arquitetura
- **[Guia de integracao MCP](./mcp-integration-guide.md)** — Referencia de ferramentas e exemplos de agentes
- **[Referencia API](./api.md)** — Endpoints REST
- **[Contribuindo](../../CONTRIBUTING.md)** — Fluxo de trabalho de desenvolvimento
