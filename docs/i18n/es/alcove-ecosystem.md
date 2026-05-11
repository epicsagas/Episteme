# Ecosistema Alcove — Analisis de arquitectura y capacidades

> Una comparacion detallada de la capa de Conocimiento Tacito de Episteme (TK-*) y el ecosistema de documentacion Alcove, cubriendo modelos de almacenamiento, capacidades de busqueda, gestion del ciclo de vida y orientacion por caso de uso.

---

## 1. Vista general de la arquitectura

### Conocimiento Tacito de Episteme (TK-*)

| Aspecto | Detalle |
|---------|---------|
| **Almacenamiento** | Archivo SQLite unico (`~/.episteme/user_knowledge.db`) |
| **Schema** | 5 tablas: `user_entities`, `user_relations`, `user_embeddings`, `user_entities_fts` (virtual FTS5), `insight_seq` |
| **Unidad** | Un insight = una fila `UserEntity` (ID TK-xxx) |
| **Grafo** | Fusionado con el grafo canonico via `CompositeGraph` en tiempo de ejecucion — permite recorrido de caminos inter-capas (TK-001 → DP-005 → SMELL-01) |
| **Concurrencia** | `Mutex<Connection>` + modo WAL para acceso MCP + CLI simultaneo |

### Sistema de documentacion Alcove

| Aspecto | Detalle |
|---------|---------|
| **Almacenamiento** | Archivos Markdown en el sistema de archivos + indice Tantivy BM25 + embeddings sqlite-vec |
| **Estructura** | Clasificacion de 3 niveles: Core (7), Suplementario (19), Publico (15) archivos por proyecto |
| **Unidad** | Un archivo Markdown estructurado (PRD, ARCHITECTURE, DECISIONS, etc.) |
| **Grafo** | Conexiones flexibles basadas en wikilinks y rutas de archivos |
| **Concurrencia** | Bloqueo basado en archivo (`.index_lock`) por raiz de docs, aislamiento de indice por vault |
| **Vaults** | 3 enlaces simbolicos a carpetas Obsidian PARA: areas (8 docs), resources (71), zettelkasten (17) |

---

## 2. Comparacion de modelos de almacenamiento

### Schema TK-* de Episteme

```sql
-- Tabla principal
CREATE TABLE user_entities (
    id TEXT PRIMARY KEY,           -- TK-001, TK-002, ...
    title TEXT,                    -- Auto: primera linea, max 80 caracteres
    content TEXT,                  -- Texto libre (sin longitud maxima)
    author TEXT DEFAULT 'user',
    confidence REAL DEFAULT 0.5,   -- +0.05 por enlace confirmado, limite 1.0
    evidence_count INTEGER DEFAULT 0,
    last_validated TEXT DEFAULT '',
    tags TEXT DEFAULT '[]',        -- Arreglo JSON
    relations TEXT DEFAULT '{}',   -- JSON HashMap<relation_type, Vec<entity_id>>
    link_provenance TEXT DEFAULT '{}',
    created_at TEXT,
    updated_at TEXT
);

-- Relaciones normalizadas (derives_from, applies_to, supersedes)
CREATE TABLE user_relations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_id TEXT,
    relation_type TEXT,
    to_id TEXT,
    UNIQUE(from_id, relation_type, to_id)
);

-- Busqueda en texto completo FTS5
CREATE VIRTUAL TABLE user_entities_fts USING fts5(title, content, tags, content=user_entities);
```

### Estructura de archivos Alcove

```
~/.alcove/
  config.toml                    # Configuracion global (docs_root, listas de archivos core/team/public, modelo de embedding)
  docs -> enlace simbolico       # → Obsidian/SecondBrain/99-Archives/projects
  vaults/
    areas -> enlace simbolico    # → Obsidian/02-Areas (8 docs)
    resources -> enlace simbolico # → Obsidian/03-Resources (71 docs)
    zettelkasten -> enlace simbolico # → Obsidian/10-Zettelkasten (17 docs)
  models/                        # Modelos de embedding ONNX en cache
  logs/

<docs_root>/<project>/
  .alcove/
    index/                       # Archivos de indice Tantivy BM25
    index_meta.json              # Huellas de archivos (mtime + tamano)
    vectors.db                   # Embeddings sqlite-vec
  PRD.md                         # Requisitos del producto
  ARCHITECTURE.md                # Diseno del sistema
  PROGRESS.md                    # Hitos y estado
  DECISIONS.md                   # Registros de decisiones arquitectonicas
  CONVENTIONS.md                 # Estandares de codificacion
  SECRETS_MAP.md                 # Variables de entorno y secretos
  DEBT.md                        # Registro de deuda tecnica
```

---

## 3. Caracter del conocimiento

| Dimension | Episteme TK-* | Alcove |
|-----------|---------------|--------|
| **Tipo** | Insights momentaneos, lecciones aprendidas, decisiones de equipo | Documentacion de proyecto estructurada (requisitos, arquitectura, decisiones) |
| **Mutabilidad** | Mutable (CRUD SQLite) | Mutable (ediciones de archivos + reconstruccion de indice) |
| **Fuente** | Texto libre contribuido por el usuario | Escrito por el usuario + generado por agente desde plantillas |
| **Autoridad** | Observacion personal/de equipo | Mandato de equipo / politica organizacional |
| **Granularidad** | Atomica (un insight por entrada) | Seccionado (multiples ADR por DECISIONS.md) |
| **Enlace** | Detectado automaticamente a entidades canonicas (scoring de palabras clave) | Wikilinks manuales + enlaces markdown |
| **Versionado** | Ninguno (solo SQLite) | Basado en Git (archivo = fuente de verdad) |

### Ciclo de vida de un insight (Episteme TK-*)

```
add_insight(text, tags?, project?, linked_entities?)
  │
  ├── Generar ID TK-xxx (secuencia atomica)
  ├── detect_canonical_links() — correspondencia de palabras clave → top 5 entidades canonicas
  │     score >= 0.5 → Enlace automatico (derives_from)
  │     score < 0.5 → Enlace sugerido
  ├── Deteccion de duplicados FTS5 → DuplicateCandidate[]
  ├── Persistir en SQLite + cache en memoria
  └── Retornar: { id, auto_links, suggested_links, duplicates, confidence }

confirm_links(id, accepted[], rejected[])
  │
  ├── Agregar relaciones derives_from/applies_to
  ├── Actualizar fuente link_provenance a "manual"
  ├── Incrementar confianza (+0.05 por enlace, limite 1.0)
  └── Persistir actualizaciones

search_insights(query, limit?)
  │
  └── Consulta FTS5 MATCH → resultados clasificados
```

### Ciclo de vida de un documento (Alcove)

```
init_project(project_name, project_path?)
  │
  ├── Crear 7 documentos core desde plantillas (PRD, ARCHITECTURE, ...)
  ├── Opcionalmente crear documentos publicos (README, CHANGELOG, ...)
  └── Reconstruir indice de busqueda

validate_docs()
  │
  ├── Verificar existencia de archivos requeridos
  ├── Verificar marcadores de plantilla (TODO, FIXME)
  ├── Verificar encabezados de seccion requeridos
  ├── Verificar cuentas minimas de elementos de lista
  └── Retornar: paso/aviso/fallo por archivo

lint_project()
  │
  ├── Detectar [[wikilinks]] y enlaces markdown rotos
  ├── Encontrar archivos huerfanos (no enlazados desde ningun documento)
  ├── Encontrar marcadores obsoletos (WIP, TODO, FIXME, DRAFT, DEPRECATED)
  └── Encontrar referencias de anos obsoletos (2+ anos)

audit_project()
  │
  ├── Escanear repo de docs privado para docs requeridos faltantes
  ├── Escanear repo de proyecto publico para docs internos expuestos
  ├── Clasificar archivos en niveles
  └── Retornar: suggested_actions[]
```

---

## 4. Capacidades de busqueda

| Capacidad | Episteme TK-* | Alcove |
|-----------|---------------|--------|
| **Motor** | FTS5 (coincidencia de palabras clave) | Tantivy BM25 + similitud coseno sqlite-vec |
| **Fusion** | Ninguna | RRF (Reciprocal Rank Fusion, k=60) |
| **CJK** | Sin soporte especial | NgramTokenizer (min=2, max=3) |
| **Fragmentacion** | N/A (una fila = un insight) | Fragmentos de 200-500 caracteres |
| **Incremental** | N/A (tabla unica) | Comparacion de huellas mtime + tamano |
| **Busqueda vectorial** | El schema existe (`user_embeddings`) pero **no conectado** | Completamente operativo (MultilingualE5Small, 384d) |
| **Alcance** | Base de datos unica | Por proyecto o global (inter-proyectos) |
| **Fallback** | Ninguno | Coincidencia de subcadena grep cuando no hay indice |

---

## 5. Completitud de funcionalidades

| Funcionalidad | Episteme TK-* | Alcove |
|---------------|---------------|--------|
| Crear | `add_insight` | `init_project`, edicion de archivos |
| Leer | `search_insights` (solo busqueda, sin obtener por ID) | `get_doc_file`, `search_project_docs` |
| Actualizar | No expuesto via MCP | Edicion directa de archivo + `rebuild_index` |
| Eliminar | No expuesto via MCP | Eliminacion de archivo + `rebuild_index` |
| Validacion | Ninguna | `validate_docs`, `lint_project` |
| Auditoria | Ninguna | `audit_project` (separacion publico/privado) |
| Respaldo | Ninguno | `backup_vault` (instantanea de commit git) |
| Importar | Ninguno | `promote_document` (Obsidian → repo de docs) |
| Politica | Ninguna | `policy.toml` con niveles de aplicacion |
| Plantillas | Ninguna | 7 core + 19 suplementarias + 15 publicas |

---

## 6. Sistema de vaults Alcove

Tres vaults, enlazados simbolicamente a la estructura PARA de Obsidian:

| Vault | Objetivo | Documentos | Proposito |
|-------|----------|------------|-----------|
| `areas` | `02-Areas` | 8 | Areas de dominio: agentes MCP, DevOps, Rust, LLM/RAG, Open Source |
| `resources` | `03-Resources` | 71 | Referencia: AWS, Leyes de Ingenieria de Software, docs tecnicos |
| `zettelkasten` | `10-Zettelkasten` | 17 | Notas atomicas: arquitectura IA, BM25, grafos de conocimiento, patrones Rust |

Cada vault tiene de forma independiente:
- Indice BM25 (Tantivy)
- Base de datos vectorial (sqlite-vec)
- Seguimiento de huellas de archivos (`index_meta.json`)
- Aislamiento de cache (separados `OnceLock<Mutex<HashMap>>`)

---

## 7. Sistema de configuracion Alcove

### Global: `~/.alcove/config.toml`

```toml
docs_root = "/ruta/a/Obsidian/SecondBrain/99-Archives/projects"

[core]
files = ["PRD.md", "ARCHITECTURE.md", "PROGRESS.md", "DECISIONS.md",
         "CONVENTIONS.md", "SECRETS_MAP.md", "DEBT.md"]

[team]
files = ["ENV_SETUP.md", "ONBOARDING.md", "DATA_MODEL.md", "SCHEMA.md",
         "DEPLOYMENT.md", "RUNBOOK.md", "PLAYBOOK.md", "MONITORING.md", ...]  # 19 archivos

[public]
files = ["README.md", "CHANGELOG.md", "CONTRIBUTING.md", "SECURITY.md", ...]  # 15 archivos

[embedding]
model = "MultilingualE5Small"
auto_download = true
enabled = true
```

### Por proyecto: `alcove.toml`

Sobrescribe los valores por defecto globales para: `diagram_format`, `core_files`, `team_files`, `public_files`.

### Politica: `policy.toml`

Define:
- Nivel `enforce`: `strict` | `warn` | `off`
- Documentos requeridos con encabezados de seccion y cuentas minimas de elementos
- Convenciones de nomenclatura (`UPPER_SNAKE`, `lower_snake`, `kebab`, `free`)
- Prioridad: proyecto > equipo > valores por defecto integrados

---

## 8. Matriz de decision por caso de uso

| Situacion | Herramienta recomendada | Justificacion |
|-----------|----------------------|---------------|
| « Registrar una leccion aprendida de un incidente en produccion » | **Episteme TK-*** | Enlaza automaticamente a smells/leyes relevantes para futuras referencias cruzadas |
| « Iniciar documentacion para un nuevo proyecto » | **Alcove** `init_project` | 7 plantillas core generadas automaticamente |
| « Verificar si hay docs desactualizados » | **Alcove** `lint_project` | Detecta automaticamente WIP/TODO/DEPRECATED/fechas obsoletas |
| « Encontrar lo que el equipo decidio sobre el middleware de auth » | **Alcove** `search_project_docs` | Busca en DECISIONS.md estructurado con BM25 + vectorial |
| « Detectar code smells en un modulo » | **Episteme** `analyze_code` | Deteccion de smells basada en patrones/regex |
| « Asegurar que el PRD tiene todas las secciones requeridas » | **Alcove** `validate_docs` | Validacion de secciones y cuentas de elementos basada en politica |
| « Vincular un insight al patron Strategy » | **Episteme** `confirm_links` | Crea una arista `derives_from` hacia la entidad canonica |
| « Importar notas de Obsidian para acceso de agentes » | **Alcove** `promote_document` | Importa al repo de docs con deteccion automatica de proyecto |
| « Encontrar la relacion entre SRP y Extract Class » | **Episteme** `find_path` | Recorrido de grafo multi-salto a traves de tipos de entidades |
| « Respaldar el estado de la documentacion del proyecto » | **Alcove** `backup_vault` | Instantanea de commit git con marca de tiempo |
| « Auditar docs internos expuestos en el repo publico » | **Alcove** `audit_project` | Escanea ubicaciones privadas y publicas |
| « Obtener sugerencias de refactoring clasificadas para el codigo » | **Episteme** `suggest_refactorings` | Scoring compuesto: severidad × esfuerzo × alineacion de principios |

---

## 9. Roles complementarios

```
Episteme TK-*                     Alcove
"¿Que principio universal         "¿Que decidio nuestro
 se aplica aqui?"                  equipo sobre esto?"

 Insight momentaneo ←────────────→ Registro de decision estructurado
 Enlace automatico por palabras    Scaffolding basado en plantillas
 clave                             Busqueda de documentos inter-proyectos
 Recorrido de grafo inter-capas    Analisis de docs → deteccion de obsolescencia
 Analisis de codigo → deteccion
   de smells
```

**Cuando ambos estan activos**: Episteme proporciona el « por que » universal (leyes, patrones), Alcove proporciona el « que decidimos » especifico del proyecto (ADR, convenciones). Los agentes deben citar ambas fuentes, con Alcove tomando precedencia cuando las reglas del equipo entran en conflicto con la orientacion generica.

---

## 10. Escala y rendimiento

| Metrica | Episteme TK-* | Alcove |
|---------|---------------|--------|
| **Capacidad disenada** | Cientos de insights | ~10 000 archivos |
| **Latencia de busqueda** | FTS5 instantaneo (en memoria) | BM25 < 500ms para vista general |
| **Eficiencia de tokens** | Un insight por resultado | Top-5 fragmentos ~1.5k tokens (vs ~8k para grep) |
| **Reconstruccion de indice** | No necesaria (disparadores FTS5) | Incremental: solo archivos modificados |
| **Tamano del modelo** | N/A (no conectado) | 15MB (ArcticEmbedXS) a 2.3GB (BGE-M3) |

---

*Ver tambien: [Guia de integracion Alcove](./alcove-integration.md) para patrones de uso y ejemplos de flujos de trabajo.*
