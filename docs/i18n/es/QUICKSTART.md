# Episteme — Guia de inicio rapido

Ponga en marcha Episteme en menos de 2 minutos.

---

## Requisitos previos

- **Rust 1.95+** (edicion 2024 requerida) — [Instalar via rustup](https://rustup.rs)
- Conexion a Internet (para la descarga inicial de datos)

---

## Opcion 1: Integracion con herramientas de IA (Recomendado)

**Ideal para:** Usuarios de Claude Code, Cursor, Codex, Gemini

```bash
# 1. Instalar Episteme
cargo install --git https://github.com/epicsagas/Episteme

# 2. Instalar en su herramienta de IA (descarga datos, configura MCP, copia agentes)
epis install claude      # Claude Code
epis install cursor      # Cursor
epis install codex       # OpenAI Codex
epis install gemini      # Antigravity
epis install all         # Todas las herramientas a la vez
```

> Si `epis install claude` falla al descargar los datos, use la instalacion desde fuentes a continuacion.

**Eso es todo.** Reinicie su herramienta de IA y Episteme estara activo.

---

## Opcion 2: Docker (No requiere Rust)

```bash
docker-compose up -d

# Acceso
# API:       http://localhost:8000
# Salud:     http://localhost:8000/health
```

Para la integracion MCP via Docker, agregue a su configuracion MCP:
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

## Opcion 3: Desde fuentes

```bash
git clone https://github.com/epicsagas/Episteme.git
cd Episteme

# Compilar
cargo build --release

# Poblar datos y construir la base de datos vectorial (la compilacion se ejecuta automaticamente)
./target/release/epis install --local
```

---

## Visualizacion del grafo

Episteme incluye un visor interactivo basado en D3-force:

```bash
episteme web               # por defecto: http://localhost:8080
episteme web --port 9001   # puerto personalizado
episteme web --host 0.0.0.0 --port 8080  # exponer en la red
```

---

## Comandos comunes

```bash
# Analizar codigo para detectar code smells
epis analyze my_code.py --language python
epis analyze my_code.py --json

# Obtener sugerencias de refactoring
episteme infer my_code.py --top-k 5

# Explorar el grafo de conocimiento
epis explore "strategy pattern"
epis graph path DP-005 RF-001

# Iniciar servidores
epis api              # API REST en :8000
episteme mcp --http       # Servidor MCP en :43175
episteme web --port 8080  # Interfaz Web

# Daemon MCP en segundo plano (proxy HTTP)
epis service start
epis service status
epis service stop

# Crear archivo de release
episteme dist --out-dir release
```

---

## Solucion de problemas

### « Base de datos no encontrada »
```bash
epis install claude   # re-descargar el archivo de datos
# o
epis install --local
```

### « Puerto ya en uso »
```bash
episteme web --port 9001
epis api --port 9000
```

---

## Proximos pasos

- **[README](../../README.md)** — Resumen completo de funcionalidades y arquitectura
- **[Guia de integracion MCP](./mcp-integration-guide.md)** — Referencia de herramientas y ejemplos de agentes
- **[Referencia API](./api.md)** — Endpoints REST
- **[Contribuir](../../CONTRIBUTING.md)** — Flujo de desarrollo
