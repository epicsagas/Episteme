<p align="center">
<img src="../assets/icon.png" alt="Episteme" width="60%" />
</p>

<p align="center"><sub>Episteme (συν ταγμα) — del griego "sistema organizado" o "discernimiento"</sub></p>

<p align="center">Un grafo de conocimiento offline-first y de unico binario que conecta patrones de diseno, tecnicas de refactoring y leyes de software a traves de relaciones semanticas.<br><b>Construido primero para agentes de IA</b> — integra la experiencia en ingenieria de software directamente en Claude Code, Cursor y otras herramientas compatibles con MCP.</p>

<p align="center">Escrito en Rust · Unico binario · Completamente offline</p>

<p align="center">
    <a href="https://github.com/epicsagas/Episteme/actions"><img src="https://img.shields.io/github/actions/workflow/status/epicsagas/Episteme/ci.yml?branch=main&label=CI" alt="CI" /></a>
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.95+-orange.svg" alt="rust-lang" /></a>
    <a href="https://crates.io/crates/episteme"><img src="https://img.shields.io/badge/crates.io-v0.1.0-orange.svg" alt="crates.io" /></a>
    <a href="../../LICENSE"><img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License" /></a>
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
  <a href="../pt/">Português</a> |
  Español |
  <a href="../hi/">हिन्दी</a>
</p>

---

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/features.png">
  <img src="../assets/features.png" align="center" width="100%" alt="Resumen de caracteristicas de Episteme" />
</picture>

---

## Inicio Rapido

### Claude Code

```
/plugin marketplace add epicsagas/plugins
/plugin install episteme@epicsagas
```

El hook del plugin instala el binario `epis` automáticamente. **Antes de iniciar una nueva sesión**, ejecuta este comando una vez en tu terminal:

```bash
epis install   # Descarga los datos del grafo de conocimiento desde GitHub Releases
```

`epis install` inicializa la base de datos del grafo de conocimiento e inicia el servidor HTTP API en el puerto 58302. Luego inicia una nueva sesión de Claude Code y estás listo.

Actualizar: `/plugin update episteme@epicsagas`

### Codex CLI

```bash
codex plugin marketplace add epicsagas/plugins
```

El hook del plugin instala el binario `epis` automáticamente. **Antes de iniciar una nueva sesión**, ejecuta este comando una vez en tu terminal:

```bash
epis install   # Descarga los datos del grafo de conocimiento desde GitHub Releases
```

`epis install` inicializa la base de datos del grafo de conocimiento e inicia el servidor HTTP API en el puerto 58302. Luego inicia una nueva sesión y estará disponible de inmediato.

Actualizar: `codex plugin update episteme@epicsagas`

### Otras herramientas

```bash
epis install cursor       # Cursor IDE
epis install opencode     # OpenCode
epis install cline        # Cline
epis install --all        # Todas las herramientas soportadas
```

### Instalacion manual

| Metodo | Comando |
|--------|---------|
| **Homebrew** | `brew install epicsagas/tap/episteme` |
| **Script shell** | `curl --proto '=https' --tlsv1.2 -LsSf https://github.com/epicsagas/Episteme/releases/latest/download/episteme-installer.sh \| sh` |
| **PowerShell** | `irm https://github.com/epicsagas/Episteme/releases/latest/download/episteme-installer.ps1 \| iex` |
| **cargo** | `cargo binstall episteme` ⚡ o `cargo install episteme` |
| **Docker** | Consulta [Opcion 3](#opcion-3-docker-no-se-requiere-rust) |

### Verificar

```bash
epis --version
epis stats
```

O desde dentro de Claude Code / Codex CLI:

```
/episteme verify
```

### Pruébalo en 30 segundos

**Opcion A — CLI:** Apuntalo a cualquier archivo de tu proyecto.

```bash
epis analyze src/domain/engine.rs
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

Episteme se activa automaticamente — no necesita sintaxis especial. Mapea tu descripcion al grafo de conocimiento y devuelve resultados clasificados y citables.

---

## Por que Episteme?

Los LLMs ya saben que es el patron Strategy. Pueden recitar los principios SOLID, listar los patrones GoF y explicar los code smells. Entonces, por que existe este proyecto?

**La brecha no es el conocimiento — es el razonamiento estructurado y conectado.**

Cuando le preguntas a un LLM "como arreglo un God Object?", te da una respuesta razonable. Pero la respuesta cambia entre conversaciones, carece de trazabilidad y no conecta el problema con sus causas raiz ni sus consecuencias posteriores. Episteme convierte hechos aislados en un grafo navegable donde cada recomendacion esta fundamentada, es citable y esta conectada al panorama de diseno mas amplio.

### En que se diferencia de simplemente hacerle un buen prompt a un LLM?

| | Prompt bien elaborado para LLM | Episteme + LLM |
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

La integracion MCP se activa automaticamente ante descripciones de problemas. Cuando un usuario dice "esta clase hace demasiadas cosas", el agente no necesita saber que preguntar sobre God Object — Episteme mapea la queja a `SMELL-03`, muestra refactorings clasificados y rastrea la violacion hasta los principios fundamentales. Esto convierte una queja vaga en un plan de remediacion estructurado.
</details>

<details>
<summary><b>2. Cuando quieres reducir el consumo de tokens — no gastarlos en explicaciones</b></summary>

Sin Episteme, un LLM responde "como arreglo un God Object?" explicando el smell, listando refactorings, describiendo los principios SOLID y repasando cada opcion — cientos de tokens por respuesta. Con Episteme, una sola llamada a herramienta MCP devuelve `SMELL-03 → RF-018 (0.89) → LAW-001`. La misma experiencia con una fraccion del presupuesto de tokens.
</details>

<details>
<summary><b>3. Cuando necesitas analisis de codigo conectado a la remediacion — no solo deteccion</b></summary>

Herramientas como SonarQube detectan smells. Los LLMs pueden sugerir patrones. Episteme hace ambas cosas y las conecta: detecta Long Method → rastrea las leyes que viola → clasifica los refactorings que lo resuelven → muestra que patrones refuerzan esos refactorings.
</details>

<details>
<summary><b>4. Cuando el conocimiento aislado de patrones no es suficiente — necesitas las relaciones</b></summary>

Saber que hace Extract Method es lo basico. Saber que *resuelve* Long Method (SMELL-01), que *viola* Single Responsibility (LAW-001), que es *reforzado por* Facade Pattern (DP-012) — esa es una cadena de razonamiento que un LLM no puede construir de forma fiable por si solo. Las 201 relaciones semanticas de Episteme permiten a los agentes de IA recorrer estos caminos de forma determinista.
</details>

<details>
<summary><b>5. Cuando estas tomando decisiones de arquitectura y necesitas evidencia, no opiniones</b></summary>

"Deberia usar microservicios?" — Episteme conecta la pregunta con la Ley de Conway (LAW-017), SRP (LAW-001) y el patron Strangler Fig (DP-026), luego muestra como se relacionan. Las decisiones se vuelven rastreables hasta leyes de ingenieria, no hasta publicaciones de blogs.
</details>

<details>
<summary><b>6. Cuando necesitas consejos de ingenieria consistentes y citables — no recomendaciones alucinadas</b></summary>

Cada hallazgo hace referencia a IDs de entidad explicitos (`DP-005`, `RF-001`, `LAW-021`). Las recomendaciones vienen con puntuaciones de prioridad y estimaciones de esfuerzo. La misma consulta siempre devuelve la misma respuesta estructurada.
</details>

<details>
<summary><b>7. Cuando trabajas en un entorno aislado o con red restringida</b></summary>

Episteme se ejecuta completamente offline: unico binario, base de datos SQLite local, embeddings locales via fastembed (ONNX Runtime). Sin telemetria, sin llamadas a servidores externos, sin API externas. Tu codigo y resultados de analisis nunca salen de tu maquina.
</details>

---

## Características

| | Característica | Por qué es importante |
|--|----------------|-----------------------|
| 🧠 | **22 Patrones de Diseño GoF** | Catálogo completo con ejemplos reales |
| 🔧 | **66 Técnicas de Refactorización** | Catálogo de Fowler con ejemplos de código |
| ⚖️ | **56 Leyes y Principios de Software** | SOLID, Ley de Conway, Teorema CAP, etc. |
| 👃 | **17 Tipos de Code Smells** | Long Method, God Object, Feature Envy, etc. ¹ |
| 🔗 | **201 Relaciones Semánticas** | "resuelve", "impone", "viola", "se relaciona con" |
| 🤖 | **9 Herramientas MCP + 4 Agentes** | Interacción de agente IA de alta fidelidad con transferencias entre agentes |
| 🌐 | **Servidor HTTP API** | API REST en el puerto 58302, se inicia automáticamente al instalar |
| 🌍 | **Soporte de 10 Lenguajes** | Python (AST), Java, TypeScript, Go, Rust, C++, C#, PHP, Ruby, Kotlin |
| 📊 | **Análisis Determinístico** | Python basado en AST + regex multilenguaje, mismo resultado siempre |
| 🏷️ | **Conocimiento Citable** | Cada hallazgo se vincula a IDs de entidad explícitos (`RF-001`, `LAW-021`) |
| 🌐 | **API REST (17 endpoints)** | Autenticación, límite de tasa, sondas de salud, métricas Prometheus |
| 📦 | **Binario Único** | Sin runtime, multiplataforma (macOS, Linux, Windows) |
| 🔌 | **Embeddings Locales** | fastembed (ONNX Runtime), búsqueda semántica sin configuración |
| 🐳 | **Soporte Docker** | Build multi-etapa con verificaciones de salud |

> ¹ Duplicate Code (SMELL-13) y Shotgun Surgery (SMELL-09) requieren contexto de múltiples archivos y se omiten en modo de archivo único.

---

## Instalacion

### Opcion 1: cargo-binstall (Recomendado)

```bash
cargo binstall episteme    # descarga binario precompilado — sin compilacion necesaria
epis install cursor        # carga datos + inicia el servidor API + instala agentes
```

Si no tienes cargo-binstall: `cargo install cargo-binstall`

> Después de `epis install`, el servidor HTTP API se inicia automáticamente en el puerto 58302. MCP sigue disponible -- consulta `registry/mcp.json` para la configuración manual.

### Opcion 2: Desde el Codigo Fuente

```bash
git clone https://github.com/epicsagas/Episteme.git
cd Episteme && cargo build --release
```

Luego ejecuta el binario para tu plataforma:

| Plataforma | Comando |
|------------|---------|
| **macOS / Linux** | `./target/release/epis install --local cursor` |
| **Windows** | `.\target\release\episteme.exe install --local cursor` |

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
    "episteme": {
      "command": "docker",
      "args": ["exec", "-i", "episteme-api", "episteme", "mcp"]
    }
  }
}
```

### Opcion 4: Binarios Precompilados (No se requiere Rust)

Descarga el ultimo binario para tu plataforma desde [GitHub Releases](https://github.com/epicsagas/Episteme/releases):

| Plataforma | Archivo |
|----------|------|
| **macOS** (Apple Silicon) | `episteme-aarch64-apple-darwin.tar.xz` |
| **Linux** (x86_64) | `episteme-x86_64-unknown-linux-gnu.tar.xz` |
| **Linux** (ARM64) | `episteme-aarch64-unknown-linux-gnu.tar.xz` |
| **Windows** (x86_64) | `episteme-x86_64-pc-windows-msvc.zip` |

```bash
# macOS / Linux
tar xzf episteme-*.tar.gz
sudo mv episteme /usr/local/bin/

# Windows — extrae el zip y agrega episteme.exe a tu PATH
```

Luego instala:
```bash
epis install cursor
```

### Verificar

```bash
epis --version
epis stats
epis explore "strategy pattern"    # explora el grafo de conocimiento
```

O desde dentro de Claude Code / Codex CLI:

```
/episteme verify
```

---

## Endpoints HTTP API

> Episteme se ejecuta como un servidor HTTP API siempre activo en el puerto 58302. Los skills y agentes usan `curl http://localhost:58302/...` en lugar de herramientas MCP. MCP sigue disponible para la configuración manual -- consulta `registry/mcp.json`.

### Endpoints de la API

#### Grafo de Conocimiento

| Método | Endpoint | Propósito |
|--------|----------|-----------|
| **GET** | `/health` | Verificación de estado |
| **GET** | `/search?q=...` | Buscar en el grafo de conocimiento |
| **GET** | `/graph/{id}` | Obtener entidad por ID |
| **GET** | `/graph/{id}/neighbors` | Obtener entidades relacionadas |
| **POST** | `/graph/path` | Encontrar ruta entre dos entidades |

#### Análisis de Código

| Método | Endpoint | Propósito |
|--------|----------|-----------|
| **POST** | `/analyze` | Detectar code smells |
| **POST** | `/refactor` | Sugerir refactorings |

#### Conocimiento Tácito

| Método | Endpoint | Propósito |
|--------|----------|-----------|
| **POST** | `/insights` | Agregar insight del equipo |

### 9 Herramientas MCP (Legacy)

#### Conocimiento canonico (6 herramientas)

| Herramienta | Proposito | Ejemplo de uso |
|------|---------|-------------|
| **`search_knowledge`** | Busqueda semantica en todas las entidades | "Buscar patrones para logica de reintentos" |
| **`get_entity`** | Obtener detalles de una entidad por ID | "Explicar Strategy Pattern (DP-023)" |
| **`get_neighbors`** | Explorar entidades relacionadas | "Que refactorings resuelven Long Method?" |
| **`find_path`** | Encontrar conexion entre dos entidades | "Como se relaciona SRP con Extract Class?" |
| **`analyze_code`** | Detectar code smells via regex/AST | "Revisar este codigo de validacion de pagos" |
| **`suggest_refactorings`** | Sugerencias de refactoring clasificadas | "Que deberia refactorizar en esta clase?" |

#### Conocimiento tacito (3 herramientas)

| Herramienta | Proposito | Ejemplo de uso |
|------|---------|-------------|
| **`add_insight`** | Registrar decisiones del equipo, lecciones aprendidas | "Elegimos event-driven sobre polling por razon X" |
| **`search_insights`** | Buscar conocimiento previo del equipo | "Que decidimos sobre el middleware de autenticacion?" |
| **`confirm_links`** | Validar enlaces detectados automaticamente a entidades canonicas | Confirmar que TK-001 se relaciona con SMELL-03 |

Episteme almacena el conocimiento tacito en una base de datos separada (`~/.episteme/user_knowledge.db`) y lo fusiona con el grafo canonico en tiempo de ejecucion mediante una capa compuesta. Las perspectivas del equipo se vinculan automaticamente con patrones, leyes y smells, convirtiendo la experiencia en conocimiento navegable.

Consulte [Arquitectura del conocimiento tacito](./tacit-knowledge.md) para el diseno completo.

### 4 Agentes Especializados (Red Conectada)

Los agentes trabajan juntos — cada analisis termina con opciones de **Proximos Pasos** que se transfieren a otros agentes.

| Agente | Cuando Usarlo | Capacidad Clave | Se transfiere a |
|--------|---------------|-----------------|-----------------|
| **`code-reviewer`** | Code smells, violaciones SOLID | Analisis de causalidad (causa raiz → sintomas derivados) | advisor, architecture-analyst, refactoring-expert |
| **`episteme-advisor`** | Decisiones de ingenieria, compromisos | Cadenas de compromisos multi-entidad con planes de accion | code-reviewer, architecture-analyst, researcher |
| **`episteme-researcher`** | Exploracion del grafo de conocimiento | Mapas de conexion entre patrones, leyes, smells | advisor, code-reviewer |
| **`architecture-analyst`** | Evaluacion de arquitectura frente a leyes | Puntuacion de cumplimiento con evaluacion ponderada por riesgo | advisor, code-reviewer, researcher |

**Ejemplo de flujo de trabajo**: `code-reviewer` detecta God Object → rastrea la causalidad hasta 3 smells derivados → ofrece "Aplicar RF-018" (→ refactoring-expert) o "Analisis profundo de causa raiz" (→ episteme-advisor) o "Verificacion de arquitectura" (→ architecture-analyst).

[Guia completa de integracion MCP](./mcp-integration-guide.md)

---

## Uso de CLI

```bash
# Analizar codigo en busca de smells
epis analyze my_code.py --language python --json
episteme infer my_code.py

# Explorar el grafo de conocimiento
epis explore "strategy pattern"
epis graph path DP-005 RF-001   # ej. Factory Method → Extract Method

# Construir el indice RAG
epis build

# Iniciar servidores
epis api              # REST API en :58302
episteme mcp --http       # Servidor MCP en :43175 (legacy)
episteme web --port 8080  # Interfaz Web (explorador de grafos interactivo)

# Empaquetado de distribucion
episteme dist --out-dir release/
```

---

## Documentacion

| Documento | Descripcion |
|-----------|-------------|
| [Inicio Rapido](./QUICKSTART.md) | Configuracion paso a paso, primera ejecucion, solucion de problemas |
| [Guia de Integracion MCP](./mcp-integration-guide.md) | Referencia de herramientas, ejemplos de agentes, flujos de conversacion |
| [Arquitectura del conocimiento tacito](./tacit-knowledge.md) | Diseno de doble base de datos, ciclo de vida de insights, esquema |
| [Comparacion del ecosistema Alcove](./alcove-ecosystem.md) | Modelos de almacenamiento, capacidades de busqueda, matriz de casos de uso |
| [Guia de integracion de Alcove](./alcove-integration.md) | Flujos de doble contexto, configuracion, buenas practicas |
| [Referencia API](./api.md) | Endpoints REST, autenticacion, ejemplos |
| [Distribucion](./distribution.md) | Empaquetado de releases y despliegue |
| [Desarrollo y Contribuciones](./DEVELOPMENT.md) | Arquitectura, como contribuir |
| [Registro de Cambios](./CHANGELOG.md) | Historial de releases y notas de version |

---

## Configuracion

### Variables de Entorno

```bash
# Ubicaciones de datos
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

## Solucion de Problemas

**Comando `episteme` no encontrado despues de instalar**

| Plataforma | Solucion |
|------------|----------|
| **macOS / Linux** | `export PATH="$HOME/.cargo/bin:$PATH"` — agrega a `~/.bashrc` o `~/.zshrc` para persistir |
| **Windows** | Agrega `%USERPROFILE%\.cargo\bin` a tu PATH del sistema, o abre una nueva terminal |

**Las herramientas MCP no aparecen en Claude Code / Cursor**

El servidor HTTP API se inicia automáticamente en el puerto 58302 después de `epis install`. Los skills usan `curl http://localhost:58302/...` para interactuar con Episteme. MCP sigue disponible para la configuración manual -- consulta `registry/mcp.json`.

**Puerto ya en uso**
```bash
epis api --port 58303   # usa un puerto diferente
```

**Primera ejecucion lenta**

Episteme construye un indice de embeddings local en la primera ejecucion. Esto tarda 30-60 segundos y es un costo unico. Las ejecuciones posteriores son instantaneas.

**Errores de compilacion durante `cargo install`**

Asegurate de tener Rust 1.95+ instalado:
```bash
rustup update stable
rustup show   # confirma el toolchain activo
```

> Mas ayuda: [seccion de solucion de problemas en QUICKSTART.md](../../QUICKSTART.md#troubleshooting) · [Abrir un issue](https://github.com/epicsagas/Episteme/issues)

---

## Hoja de Ruta

**Publicado**
- [x] `epis install` — configuración de datos con un solo comando desde GitHub Releases
- [x] Homebrew tap (`epicsagas/tap/episteme`) — macOS Apple Silicon + Linux (x86_64 + ARM64)
- [x] Soporte del marketplace de plugins de Claude Code & Codex CLI
- [x] Traducciones del README — 9 idiomas (ko, ja, zh-CN, zh-TW, de, fr, es, pt, hi)
- [x] **Compilaciones multiplataforma** — macOS, Linux, Windows (con aceleración GPU DirectML)

**Planificado**
- [ ] **Entidades Personalizadas** — Agregar patrones/smells específicos del equipo
- [ ] **Metadatos Multilingüe** — Títulos y resúmenes de entidades en idiomas CJK
- [ ] **Tutoriales Interactivos** — Guías integradas en la aplicación para herramientas MCP
- [ ] **Métricas de Equipo** — Uso agregado de patrones a nivel organizacional

---

## Contribuciones

Las contribuciones son bienvenidas! Consulta [DEVELOPMENT.md](./DEVELOPMENT.md) para el resumen de arquitectura y la guia de contribucion.

```bash
# Ejecutar pruebas
cargo test

# Lint
cargo clippy -- -D warnings

# Formato
cargo fmt
```

Preguntas? [Abre una discusion](https://github.com/epicsagas/Episteme/discussions) o [crea un issue](https://github.com/epicsagas/Episteme/issues).

---

## Licencia

Apache 2.0 — consulta [LICENSE](../../LICENSE) para mas detalles.
