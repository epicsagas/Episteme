<h1 align="center">Syntagma</h1>

<p align="center"><b>Grafo de Conocimiento para Ingenieria de Software</b></p>

<p align="center"><sub>Syntagma (συν ταγμα) — del griego "sistema organizado" o "discernimiento"</sub></p>

<p align="center">Un grafo de conocimiento offline-first y de unico binario que conecta patrones de diseno, tecnicas de refactoring y leyes de software a traves de relaciones semanticas.<br><b>Construido primero para agentes de IA</b> — integra la experiencia en ingenieria de software directamente en Claude Code, Cursor y otras herramientas compatibles con MCP.</p>

<p align="center">Escrito en Rust · Unico binario · Completamente offline</p>

<p align="center">
    <a href="https://github.com/epicsagas/Syntagma/actions"><img src="https://img.shields.io/github/actions/workflow/status/epicsagas/Syntagma/ci.yml?branch=main&label=CI" alt="CI" /></a>
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.95+-orange.svg" alt="rust-lang" /></a>
    <a href="https://crates.io/crates/syntagma"><img src="https://img.shields.io/badge/crates.io-v0.1.0-orange.svg" alt="crates.io" /></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License" /></a>
    <a href="https://buymeacoffee.com/epicsaga"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-FFDD00?style=flat&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me a Coffee" /></a>
</p>

<p align="center">
  <a href="../../README.md">English</a> |
  <a href="README.ja.md">日本語</a> |
  <a href="README.ko.md">한국어</a> |
  <a href="README.de.md">Deutsch</a> |
  <a href="README.fr.md">Français</a> |
  <a href="README.zh-CN.md">简体中文</a> |
  <a href="README.zh-TW.md">繁體中文</a> |
  <a href="README.pt.md">Português</a> |
  Español |
  <a href="README.hi.md">हिन्दी</a>
</p>



---

<img src="../assets/features.png" align="center" width="100%" alt="Resumen de caracteristicas de Syntagma" />

---

## Inicio Rapido

> **Requisitos previos:** Rust 1.95+ via [rustup](https://rustup.rs) · **No tienes Rust?** Consulta [Docker](#opcion-3-docker-no-se-requiere-rust) o [binarios precompilados](#opcion-4-binarios-precompilados-no-se-requiere-rust).

**1. Instalar Rust (si aun no esta instalado)**

| SO | Comando |
|----|---------|
| **macOS / Linux** | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **Windows** | Descarga y ejecuta [`rustup-init.exe`](https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe) |

Despues de instalar, abre una **nueva terminal** (o ejecuta `source "$HOME/.cargo/env"` en macOS/Linux).

**2. Instalar Syntagma (la primera compilacion tarda 3-5 min)**

```bash
cargo install --git https://github.com/epicsagas/Syntagma
```

**3. Cargar datos y conectar tu herramienta de IA**

```bash
syntagma install claude    # o: cursor, codex, gemini
```

**4. Verificar**

```bash
syntagma --version
syntagma stats
```

Eso es todo. Reinicia Claude Code y las herramientas de Syntagma estaran listas.

### Pruébalo en 30 segundos

**Opcion A — CLI:** Apuntalo a cualquier archivo de tu proyecto.

```bash
syntagma analyze src/domain/engine.rs
```

```
✓ 2 smells detectados en src/domain/engine.rs

  SMELL-07 (Large Class) — RefactoringRanker, 743 lineas
  → RF-018 Extract Class          prioridad 0.89  esfuerzo: medio
  → RF-001 Extract Method         prioridad 0.76  esfuerzo: pequeno
  → Viola: LAW-001 Principio de Responsabilidad Unica

  SMELL-01 (Long Method) — rank_refactorings(), 58 lineas
  → RF-001 Extract Method         prioridad 0.92  esfuerzo: pequeno
  → Viola: LAW-001 SRP, LAW-004 DRY
```

**Opcion B — Claude Code:** Abre cualquier archivo de tu proyecto y pregunta de forma natural.

```
Encuentra code smells en este proyecto y sugiere refactorings.
```

Syntagma se activa automaticamente — no necesita sintaxis especial. Mapea tu descripcion al grafo de conocimiento y devuelve resultados clasificados y citables.

---

## Por que Syntagma?

Los LLMs ya saben que es el patron Strategy. Pueden recitar los principios SOLID, listar los patrones GoF y explicar los code smells. Entonces, por que existe este proyecto?

**La brecha no es el conocimiento — es el razonamiento estructurado y conectado.**

Cuando le preguntas a un LLM "como arreglo un God Object?", te da una respuesta razonable. Pero la respuesta cambia entre conversaciones, carece de trazabilidad y no conecta el problema con sus causas raiz ni sus consecuencias posteriores. Syntagma convierte hechos aislados en un grafo navegable donde cada recomendacion esta fundamentada, es citable y esta conectada al panorama de diseno mas amplio.

### En que se diferencia de simplemente hacerle un buen prompt a un LLM?

| | Prompt bien elaborado para LLM | Syntagma + LLM |
|---|---|---|
| Deteccion proactiva | Solo si el usuario hace la pregunta correcta | Se activa automaticamente ante descripciones de problemas |
| Eficiencia de tokens | Explicaciones largas + multiples turnos de seguimiento | Una sola llamada a herramienta devuelve un resultado estructurado |
| Recorrido de relaciones | Un salto como maximo, frecuentemente alucinado | Recorrido de grafos multihop, verificado |
| Referencia cruzada | Manual, propensa a errores | Automatica a traves de 201 relaciones semanticas |
| Consistencia | Varia entre conversaciones | La misma respuesta estructurada cada vez |
| Citabilidad | "Creo que deberias usar Extract Class" | "Extract Class (RF-018), prioridad 0.89" |
| Offline / Aislado | Requiere internet para mejores resultados | Completamente local, unico binario |

### Cuando es util?

<details>
<summary><b>1. Cuando tu agente de IA deberia detectar problemas proactivamente, no esperar a que se lo pidan</b></summary>

La integracion MCP se activa automaticamente ante descripciones de problemas. Cuando un usuario dice "esta clase hace demasiadas cosas", el agente no necesita saber que preguntar sobre God Object — Syntagma mapea la queja a `SMELL-03`, muestra refactorings clasificados y rastrea la violacion hasta los principios fundamentales. Esto convierte una queja vaga en un plan de remediacion estructurado.
</details>

<details>
<summary><b>2. Cuando quieres reducir el consumo de tokens — no gastarlos en explicaciones</b></summary>

Sin Syntagma, un LLM responde "como arreglo un God Object?" explicando el smell, listando refactorings, describiendo los principios SOLID y repasando cada opcion — cientos de tokens por respuesta. Con Syntagma, una sola llamada a herramienta MCP devuelve `SMELL-03 → RF-018 (0.89) → LAW-001`. La misma experiencia con una fraccion del presupuesto de tokens.
</details>

<details>
<summary><b>3. Cuando necesitas analisis de codigo conectado a la remediacion — no solo deteccion</b></summary>

Herramientas como SonarQube detectan smells. Los LLMs pueden sugerir patrones. Syntagma hace ambas cosas y las conecta: detecta Long Method → rastrea las leyes que viola → clasifica los refactorings que lo resuelven → muestra que patrones refuerzan esos refactorings.
</details>

<details>
<summary><b>4. Cuando el conocimiento aislado de patrones no es suficiente — necesitas las relaciones</b></summary>

Saber que hace Extract Method es lo basico. Saber que *resuelve* Long Method (SMELL-01), que *viola* Single Responsibility (LAW-001), que es *reforzado por* Facade Pattern (DP-012) — esa es una cadena de razonamiento que un LLM no puede construir de forma fiable por si solo. Las 201 relaciones semanticas de Syntagma permiten a los agentes de IA recorrer estos caminos de forma determinista.
</details>

<details>
<summary><b>5. Cuando estas tomando decisiones de arquitectura y necesitas evidencia, no opiniones</b></summary>

"Deberia usar microservicios?" — Syntagma conecta la pregunta con la Ley de Conway (LAW-017), SRP (LAW-001) y el patron Strangler Fig (DP-026), luego muestra como se relacionan. Las decisiones se vuelven rastreables hasta leyes de ingenieria, no hasta publicaciones de blogs.
</details>

<details>
<summary><b>6. Cuando necesitas consejos de ingenieria consistentes y citables — no recomendaciones alucinadas</b></summary>

Cada hallazgo hace referencia a IDs de entidad explicitos (`DP-005`, `RF-001`, `LAW-021`). Las recomendaciones vienen con puntuaciones de prioridad y estimaciones de esfuerzo. La misma consulta siempre devuelve la misma respuesta estructurada.
</details>

<details>
<summary><b>7. Cuando trabajas en un entorno aislado o con red restringida</b></summary>

Syntagma se ejecuta completamente offline: unico binario, base de datos SQLite local, embeddings locales via fastembed (ONNX Runtime). Sin telemetria, sin llamadas a servidores externos, sin API externas. Tu codigo y resultados de analisis nunca salen de tu maquina.
</details>

---

## Instalacion

### Opcion 1: Un Solo Comando (Recomendado)

```bash
# La primera compilacion tarda 3-5 minutos — esto es normal
cargo install --git https://github.com/epicsagas/Syntagma
syntagma install claude    # carga datos + configura MCP + instala agentes
```

> Despues de `syntagma install claude`, **reinicia Claude Code** para que aparezcan las herramientas y agentes MCP.

### Opcion 2: Desde el Codigo Fuente

```bash
git clone https://github.com/epicsagas/Syntagma.git
cd Syntagma && cargo build --release
```

Luego ejecuta el binario para tu plataforma:

| Plataforma | Comando |
|------------|---------|
| **macOS / Linux** | `./target/release/syntagma install --local claude` |
| **Windows** | `.\target\release\syntagma.exe install --local claude` |

### Opcion 3: Docker (No se requiere Rust)

```bash
docker-compose up -d
```

Agrega a tu archivo de configuracion MCP:

| Herramienta | Ruta del archivo de configuracion |
|-------------|----------------------------------|
| Claude Code | `~/.claude.json` |
| Cursor | `.cursor/mcp.json` |
| VS Code (Copilot) | `.vscode/mcp.json` |

```json
{
  "mcpServers": {
    "syntagma": {
      "command": "docker",
      "args": ["exec", "-i", "syntagma-api", "syntagma", "mcp"]
    }
  }
}
```

### Opcion 4: Binarios Precompilados (No se requiere Rust)

Descarga el ultimo binario para tu plataforma desde [GitHub Releases](https://github.com/epicsagas/Syntagma/releases):

| Plataforma | Archivo |
|------------|---------|
| **macOS** (Apple Silicon) | `syntagma-aarch64-apple-darwin.tar.gz` |
| **macOS** (Intel) | `syntagma-x86_64-apple-darwin.tar.gz` |
| **Linux** (x86_64) | `syntagma-x86_64-unknown-linux-gnu.tar.gz` |
| **Linux** (ARM64) | `syntagma-aarch64-unknown-linux-gnu.tar.gz` |
| **Windows** (x86_64) | `syntagma-x86_64-pc-windows-msvc.zip` |

```bash
# macOS / Linux
tar xzf syntagma-*.tar.gz
sudo mv syntagma /usr/local/bin/

# Windows — extrae el zip y agrega syntagma.exe a tu PATH
```

Luego instala:
```bash
syntagma install claude    # o: cursor, codex, gemini
```

### Verificar

```bash
syntagma --version
syntagma stats
syntagma explore "strategy pattern"    # explora el grafo de conocimiento
```

---

## Herramientas MCP y Agentes

> **Que es MCP?** El [Model Context Protocol](https://modelcontextprotocol.io) es un estandar abierto que permite a las herramientas de IA llamar a servicios externos. Syntagma expone su grafo de conocimiento como herramientas MCP que Claude Code, Cursor y otros editores compatibles pueden llamar automaticamente.

### 6 Herramientas MCP

| Herramienta | Proposito | Ejemplo de Uso |
|-------------|-----------|----------------|
| **`search_knowledge`** | Busqueda semantica en todas las entidades | "Buscar patrones para logica de reintentos" |
| **`get_entity`** | Obtener detalles de una entidad especifica por ID | "Explicar el patron Strategy (DP-023)" |
| **`get_neighbors`** | Explorar entidades relacionadas | "Que refactorings resuelven Long Method?" |
| **`find_path`** | Encontrar conexion entre dos entidades | "Como se relaciona SRP con Extract Class?" |
| **`analyze_code`** | Detectar code smells via analisis regex/AST | "Revisar este codigo de validacion de pagos" |
| **`suggest_refactorings`** | Sugerencias de refactoring clasificadas | "Que deberia refactorizar en esta clase?" |

### 4 Agentes Especializados (Red Conectada)

Los agentes trabajan juntos — cada analisis termina con opciones de **Proximos Pasos** que se transfieren a otros agentes.

| Agente | Cuando Usarlo | Capacidad Clave | Se transfiere a |
|--------|---------------|-----------------|-----------------|
| **`code-reviewer`** | Code smells, violaciones SOLID | Analisis de causalidad (causa raiz → sintomas derivados) | advisor, architecture-analyst, refactoring-expert |
| **`syntagma-advisor`** | Decisiones de ingenieria, compromisos | Cadenas de compromisos multi-entidad con planes de accion | code-reviewer, architecture-analyst, researcher |
| **`syntagma-researcher`** | Exploracion del grafo de conocimiento | Mapas de conexion entre patrones, leyes, smells | advisor, code-reviewer |
| **`architecture-analyst`** | Evaluacion de arquitectura frente a leyes | Puntuacion de cumplimiento con evaluacion ponderada por riesgo | advisor, code-reviewer, researcher |

**Ejemplo de flujo de trabajo**: `code-reviewer` detecta God Object → rastrea la causalidad hasta 3 smells derivados → ofrece "Aplicar RF-018" (→ refactoring-expert) o "Analisis profundo de causa raiz" (→ syntagma-advisor) o "Verificacion de arquitectura" (→ architecture-analyst).

[Guia completa de integracion MCP](docs/mcp-integration-guide.md)

---

## Uso de CLI

```bash
# Analizar codigo en busca de smells
syntagma analyze my_code.py --language python --json
syntagma infer my_code.py

# Explorar el grafo de conocimiento
syntagma explore "strategy pattern"
syntagma graph path DP-005 RF-001   # ej. Factory Method → Extract Method

# Construir el indice RAG
syntagma build

# Iniciar servidores
syntagma api              # REST API en :8000
syntagma mcp --http       # Servidor MCP en :43175
syntagma web --port 8080  # Interfaz Web (explorador de grafos interactivo)

# Empaquetado de distribucion
syntagma dist --out-dir release/
```

---

## Caracteristicas

### Base de Conocimiento
- **22 Patrones de Diseno GoF** — Catalogo completo con ejemplos del mundo real
- **66 Tecnicas de Refactoring** — Del catalogo de Fowler con ejemplos de codigo
- **56 Leyes y Principios de Software** — SOLID, Ley de Conway, Teorema CAP, etc.
- **17 Tipos de Code Smells** — Long Method, God Object, Feature Envy, etc. ¹
- **201 Relaciones Semanticas** — "resuelve", "refuerza", "viola", "se_relaciona_con"

### Diseno AI-First
- **Integracion MCP** — 6 herramientas especializadas para interaccion de alta fidelidad con agentes de IA
- **4 Agentes Conectados** — Analisis de causalidad, seguimientos interactivos y transferencias entre agentes
- **Soporte para 10 Lenguajes** — Python (AST), Java, TypeScript, Go, Rust, C++, C#, PHP, Ruby, Kotlin
- **Analisis Determinista** — Deteccion basada en AST para Python + soporte multilenguaje basado en regex
- **Conocimiento Citable** — Cada hallazgo se vincula a IDs de entidad explicitos (ej., `RF-001`, `LAW-021`)
- **Cadenas de Flujo de Trabajo** — Pipelines de multiples pasos: Revision de Codigo → Analisis de Causalidad → Refactoring → Verificacion

### Listo para Produccion
- **REST API** — 17 endpoints con autenticacion y limitacion de tasa
- **Unico Binario** — Sin dependencias de tiempo de ejecucion, multiplataforma
- **Embeddings Locales** — fastembed (ONNX Runtime) para busqueda semantica sin configuracion
- **Visualizacion Interactiva** — Explorador de grafos basado en web (`syntagma web`)
- **Soporte Docker** — Build multi-etapa con health checks
- **Monitoreo** — Endpoint de metricas Prometheus

> ¹ Duplicate Code (SMELL-13) y Shotgun Surgery (SMELL-09) requieren contexto de multiples archivos y se omiten en modo de archivo unico.

---

## Documentacion

| Documento | Descripcion |
|-----------|-------------|
| [Inicio Rapido](QUICKSTART.md) | Configuracion paso a paso, primera ejecucion, solucion de problemas |
| [Guia de Integracion MCP](docs/mcp-integration-guide.md) | Referencia de herramientas, ejemplos de agentes, flujos de conversacion |
| [Referencia API](docs/api.md) | Endpoints REST, autenticacion, ejemplos |
| [Distribucion](docs/distribution.md) | Empaquetado de releases y despliegue |
| [Desarrollo y Contribuciones](DEVELOPMENT.md) | Arquitectura, como contribuir |
| [Registro de Cambios](CHANGELOG.md) | Historial de releases y notas de version |

---

## Configuracion

### Variables de Entorno

```bash
# Ubicaciones de datos
SYNTAGMA_DATA_DIR=~/.syntagma/data
SYNTAGMA_DB_PATH=~/.syntagma/db/syntagma.db

# Servidor API
SYNTAGMA_API_HOST=0.0.0.0
SYNTAGMA_API_PORT=8000
SYNTAGMA_API_KEY=your-secret-key

# Servidor MCP
SYNTAGMA_MCP_HOST=127.0.0.1
SYNTAGMA_MCP_PORT=43175
```

---

## Solucion de Problemas

**Comando `syntagma` no encontrado despues de instalar**

| Plataforma | Solucion |
|------------|----------|
| **macOS / Linux** | `export PATH="$HOME/.cargo/bin:$PATH"` — agrega a `~/.bashrc` o `~/.zshrc` para persistir |
| **Windows** | Agrega `%USERPROFILE%\.cargo\bin` a tu PATH del sistema, o abre una nueva terminal |

**Las herramientas MCP no aparecen en Claude Code / Cursor**

Reinicia el editor despues de ejecutar `syntagma install`. Si aun no aparecen, verifica que la configuracion se haya escrito:
```bash
cat ~/.claude.json   # Claude Code
```

**Puerto ya en uso**
```bash
syntagma mcp --http --port 43176   # usa un puerto diferente
```

**Primera ejecucion lenta**

Syntagma construye un indice de embeddings local en la primera ejecucion. Esto tarda 30-60 segundos y es un costo unico. Las ejecuciones posteriores son instantaneas.

**Errores de compilacion durante `cargo install`**

Asegurate de tener Rust 1.95+ instalado:
```bash
rustup update stable
rustup show   # confirma el toolchain activo
```

> Mas ayuda: [seccion de solucion de problemas en QUICKSTART.md](QUICKSTART.md#troubleshooting) · [Abrir un issue](https://github.com/epicsagas/Syntagma/issues)

---

## Hoja de Ruta

- [ ] **Tutoriales Interactivos** — Guias integradas en la aplicacion para herramientas MCP
- [ ] **Metricas de Equipo** — Uso agregado de patrones a nivel organizacional
- [ ] **Entidades Personalizadas** — Agregar patrones/smells especificos del equipo
- [ ] **Plugins IDE** — Integraciones nativas para VSCode, IntelliJ
- [ ] **Documentacion Multilingue** — Base de conocimiento en coreano, japones, chino

---

## Contribuciones

Las contribuciones son bienvenidas! Consulta [DEVELOPMENT.md](DEVELOPMENT.md) para el resumen de arquitectura y la guia de contribucion.

```bash
# Ejecutar pruebas
cargo test

# Lint
cargo clippy -- -D warnings

# Formato
cargo fmt
```

Preguntas? [Abre una discusion](https://github.com/epicsagas/Syntagma/discussions) o [crea un issue](https://github.com/epicsagas/Syntagma/issues).

---

## Licencia

Apache 2.0 — consulta [LICENSE](LICENSE) para mas detalles.
