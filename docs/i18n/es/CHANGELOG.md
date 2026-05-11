# Registro de cambios

Todos los cambios notables de Episteme seran documentados en este archivo.

El formato esta basado en [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
y este proyecto se adhiere a [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Sin publicar]

### Modificado

- CLI: `explore` renombrado a `search` (el nombre anterior funciona como alias deprecado)
- CLI: `mcp` y `api` ahora gestionan su ciclo de vida completo de servicios (`start`, `stop`, `restart`, `status`, `enable [--now]`, `disable [--now]`)
- CLI: el comando `service` de nivel superior esta deprecado — use `mcp start/stop/restart/status/enable/disable` en su lugar
- CLI: `mcp --http` esta deprecado — use `mcp start` para el modo daemon HTTP
- CLI: `launchd-install/uninstall/status` esta deprecado — use `mcp enable/disable/status` en su lugar
- `enable/disable` ahora multiplataforma: macOS (launchd) y Linux (unidad de usuario systemd)

### Agregado

- `api start/stop/restart/status/enable/disable` — gestion del ciclo de vida del daemon API REST
- Generacion de unidades de usuario systemd Linux para `mcp enable`

- **Transporte MCP HTTP para Claude Code** — selector de transporte TUI, HTTP por defecto, activacion automatica launchd
- **Instalacion automatica de prompts de agente** — `epis install` copia los prompts de agente Episteme en `~/.claude/agents/`
- **Descripciones de entidades** — campo descripcion extraido automaticamente de archivos fuente markdown, mostrado en el panel de detalles del visor web
- **SPA de visualizacion de benchmarks** — analisis de tendencias, panel de desglose de consultas
- **Rediseno del visor web** — diagrama Sankey, arbol en barra lateral, panel de detalles, mejoras de legibilidad de subgrafos
- **Upsert de configuracion MCP** — re-ejecutar `epis install` actualiza el transporte cuando la configuracion difiere (stdio ↔ HTTP)
- **Configuracion MCP YAML** — `mcp.host` / `mcp.port` en `config.yaml` (yaml → fallback a variable de entorno)
- **Monitoreo** — soporte nativo y remoto de scrape Prometheus via variables de entorno
- **Endurecimiento CI** — cargo audit, gitleaks, generacion SBOM, SHA de acciones fijados
- **Pipeline de release** — objetivo Windows, publicacion en crates.io, tap Homebrew
- **Ejemplo de diagnostico arquitectonico de modulo Dios** en `examples/`

### Modificado

- **Asistente de instalacion** — todos los pasos (transporte, Redis, telemetria) migrados a TUI de pantalla completa
- **Flujo de instalacion** — construye automaticamente el indice RAG despues de poblar, omite cuando la base ya existe
- **Grafo de conocimiento** — enriquecido con relaciones semanticas inter-entidades
- **Licencia** — MIT → Apache-2.0

### Corregido

- Panic del runtime Tokio en `main()` sincrono para telemetria
- Calidad de busqueda — bug de medicion NDCG resuelto, precision hit@1 mejorada al 100 %
- Recall de busqueda — boosting inter-tipos, manejo de entidades dispersas, sinonimos de intencion
- Cache del modelo fastembed fijado a `~/.episteme/models`
- Sustitucion de UID de bootstrap launchd y manejo de puerto en uso
- Origines CORS ahora configurables via `EPISTEME_CORS_ORIGINS`

## [0.1.0] - 2026-05-03

### Agregado

- **Reescritura completa en Rust** — reemplazo total de la base de codigo Python con Rust idiomático
- **Arquitectura hexagonal** — `ports/` (traits), `domain/` (logica de negocio), `adapters/` (infraestructura), `server/` (HTTP)
- **Framework GenericParser** — 8 parsers basados en llaves consolidados en `GenericParser` con `ParserConfig`; patrones regex cacheados via `OnceLock` con `Box::leak`
- **Parsing AST Python** — `rustpython-parser` para deteccion precisa de code smells Python (Long Method, Large Class, God Object)
- **TieredAccum + build_detection()** — deduplicacion de 14 construcciones identicas de deteccion de smells en `detectors.rs` (1 253 → 591 lineas)
- **Descomposicion del modulo MCP** — separacion de `EpistemeMCP` (675 lineas) en servicios `mcp_search`, `mcp_graph`, `mcp_analysis`
- **Descomposicion de comandos CLI** — separacion de `main.rs` (1 741 lineas) en modulo `commands/` con `cli.rs` para definiciones clap
- **Deduplicacion de manejadores API** — fusion de duplicados `search`/`search_post` en `do_search()` compartido
- **16 funciones de detectores de smells** — frente a 14 anteriormente, cubriendo todas las categorias de smells GoF
- **17 endpoints API REST** — sondas de salud, metricas Prometheus, CORS, limitacion de tasa
- **Eviccion TTL del limitador de tasa** — MAX_BUCKETS=10 000 con TTL de 1 hora para evitar crecimiento de memoria no limitado
- **Mitigacion ReDoS** — regex de operador ternario acotada de `[^:]+` a `[^:\n]{1,50}`
- **Embeddings locales** — fastembed (ONNX Runtime) para busqueda semantica sin configuracion
- **Asistente de instalacion interactivo** — TUI con crossterm, atajos vim, pantalla alternativa
- **Empaquetado de distribucion** — comando `episteme dist` para creacion de archivos de release con bootstrap automatico de base
- **CI multiplataforma** — workflow de release GitHub Actions para linux/macOS (x86_64 + aarch64)
- **Dockerfile multi-etapa** — compilador Rust + runtime Debian ligero

### Modificado

- **Lenguaje**: Python 3.11+ → Rust (edicion 2024)
- **Framework Web**: FastAPI → axum
- **Base de datos**: Python sqlite3 → rusqlite (incluido)
- **Embeddings**: sentence-transformers/PyTorch → fastembed/ONNX Runtime
- **CLI**: argparse → clap (derive)
- **Todos los patrones regex cacheados** — cero recompilacion en rutas criticas via `REGEX_CACHE` global

### Eliminado

- Dependencia del runtime Python
- Dependencia de ChromaDB
- Dependencia de tree-sitter
- Workflow de publicacion PyPI
- Binario autonomo `episteme-hook` (era punto de entrada PyPI solo Python) — use `episteme hooks ground|sniff|audit` en su lugar

## [0.0.5] - 2026-04-30

### Agregado

- Interfaz web de visualizacion del grafo (`episteme web`) con D3-force
- Base vectorial pre-construida en el archivo de release
- Flag `epis install --local` para flujos de desarrollo
- 650+ relaciones semanticas cubriendo las 161 entidades
- Generacion automatica de base vectorial en CI durante releases

## [0.0.4] - 2026-04-29

### Agregado

- Servidor MCP con 6 herramientas
- 4 agentes especializados
- Comando `epis install`
- Gestion de daemon `epis service`
- Busqueda hibrida (FTS5 + vectorial)
- Cache Redis, aceleracion GPU
- Deteccion de code smells en 10 lenguajes
- Monitoreo Prometheus + Grafana
