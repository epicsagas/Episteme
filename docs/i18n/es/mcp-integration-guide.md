# Guia de integracion MCP

> Integre el grafo de conocimiento de Episteme en Claude Code, Cursor y otras herramientas de IA compatibles con MCP

## Modo HTTP MCP de Rust (Actual)
Use el transporte HTTP independiente directamente:

```bash
# Iniciar MCP sobre HTTP
episteme mcp --http --host 127.0.0.1 --port 43175
```

Comportamiento de autenticacion:
- Si `EPISTEME_API_KEYS` esta configurado, las solicitudes deben incluir:
```http
Authorization: Bearer <api-key>
```
- Si no hay claves configuradas, la autenticacion se omite (modo desarrollo).
- `GET /health` siempre es publico para verificaciones de estado.

Nota:
- `epis service` gestiona este mismo modo HTTP MCP en segundo plano (`start|stop|status|enable|disable`).
- Los ejemplos antiguos con `--proxy` estan obsoletos; use `mcp --http`/`service` directamente.

## ¿Que es MCP?

[Model Context Protocol (MCP)](https://modelcontextprotocol.io) es un estandar abierto que permite a los asistentes de IA acceder a herramientas y fuentes de datos externas. Episteme proporciona 6 herramientas MCP que dan a los agentes de IA acceso directo al conocimiento de ingenieria de software.

---

## Inicio rapido (Claude Code)

### 1. Instalar Episteme

```bash
# Instalar (requiere Rust 1.95+)
cargo install --git https://github.com/epicsagas/Episteme

# Instalar agentes y servidor MCP en Claude Code
# (prepara datos y configura MCP automaticamente)
epis install claude
```

> Si la descarga de datos falla, use instalacion desde fuente: `git clone` → `cargo build --release` → `epis install --local`

### 2. Verificar la instalacion

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

### 3. Comenzar a usar

Reinicie Claude Code. Ahora tiene acceso a 6 herramientas de Episteme:

```
Usuario: "¿Cual es la mejor manera de corregir un smell God Object?"

Claude (usando la herramienta search_knowledge):
  → Busca refactorizaciones de "God Object"
  → Retorna: RF-018 (Extract Class), RF-023 (Move Method)
  
Claude: "El anti-patron God Object (SMELL-03) viola el Principio de 
Responsabilidad Unica (LAW-001). Mejores refactorizaciones:

1. Extract Class (RF-018) - Mover metodos/campos relacionados a una nueva clase
2. Move Method (RF-023) - Reubicar metodos a las clases apropiadas

Ambos aplican los principios SOLID y mejoran la testeabilidad."
```

---

## Referencia de herramientas MCP

### 1. `search_knowledge`

**Proposito**: Busqueda semantica en todas las entidades (patrones, leyes, refactorizaciones, smells)

**Parametros**:
```typescript
{
  query: string          // Consulta en lenguaje natural
  top_k?: number         // Resultados a retornar (por defecto: 5)
  filter_type?: string   // "pattern", "law", "refactoring", "smell"
}
```

**Retorna**:
```typescript
{
  results: [{
    entity_id: string     // ej: "DP-023"
    title: string
    entity_type: string
    similarity: number    // 0.0-1.0
    summary: string
  }]
}
```

**Ejemplo de conversacion**:
```
Usuario: "¿Como hago mi codigo mas testeable?"

Claude llama: search_knowledge({
  query: "improve testability",
  top_k: 3
})

Retorna:
- LAW-001: Single Responsibility Principle
- DP-018: Dependency Injection
- RF-042: Extract Interface

Claude: "Tres enfoques clave para mejorar la testeabilidad:
1. Aplicar SRP (LAW-001) - Una clase, una razon para cambiar
2. Usar Dependency Injection (DP-023) - Inyectar dependencias
3. Extract Interface (RF-042) - Simular dependencias externas"
```

---

### 2. `get_entity`

**Proposito**: Obtener detalles completos de una entidad especifica por ID

**Parametros**:
```typescript
{
  entity_id: string   // ej: "DP-023", "RF-001", "SMELL-01"
}
```

**Retorna**:
```typescript
{
  entity_id: string
  title: string
  type: string
  description: string
  implementation: string    // Ejemplos de codigo
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

**Ejemplo de conversacion**:
```
Usuario: "Explica Strategy Pattern en detalle"

Claude llama: get_entity({ entity_id: "DP-023" })

Retorna:
- Descripcion completa de Strategy Pattern
- Cuando usar (multiples algoritmos, cambio en tiempo de ejecucion)
- Implementacion (Context, interfaz Strategy, ConcreteStrategies)
- Beneficios: Principio Open/Closed, flexibilidad en tiempo de ejecucion
- Compromisos: Mayor cantidad de clases
- Relacionados: LAW-002 (Open/Closed), RF-034 (Replace Conditional with Polymorphism)

Claude: "Strategy Pattern (DP-023) encapsula algoritmos en clases separadas...
[proporciona explicacion detallada con ejemplo de codigo]

Conceptos relacionados:
- Aplica el Principio Open/Closed (LAW-002)
- A menudo se aplica via Replace Conditional with Polymorphism (RF-034)"
```

---

### 3. `get_neighbors`

**Proposito**: Explorar entidades conectadas a una entidad dada

**Parametros**:
```typescript
{
  entity_id: string
  relation_type?: string   // "solves", "enforces", "violates", "relates_to"
  max_depth?: number       // Por defecto: 1
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

**Ejemplo de conversacion**:
```
Usuario: "¿Que refactorizaciones resuelven el smell Long Method?"

Claude llama: get_neighbors({
  entity_id: "SMELL-01",
  relation_type: "solved_by"
})

Retorna:
- RF-001: Extract Method
- RF-034: Decompose Conditional
- RF-056: Replace Temp with Query

Claude: "Long Method (SMELL-01) puede resolverse con:
1. Extract Method (RF-001) - Dividir en metodos mas pequenos
2. Decompose Conditional (RF-034) - Simplificar condiciones complejas
3. Replace Temp with Query (RF-056) - Eliminar variables temporales"
```

---

### 4. `find_path`

**Proposito**: Encontrar la conexion entre dos entidades (ej: como una ley se relaciona con un patron)

**Parametros**:
```typescript
{
  from_id: string
  to_id: string
  max_depth?: number   // Por defecto: 5
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

**Ejemplo de conversacion**:
```
Usuario: "¿Como se relaciona el Principio de Responsabilidad Unica con la refactorizacion Extract Class?"

Claude llama: find_path({
  from_id: "LAW-001",
  to_id: "RF-018"
})

Retorna la ruta:
LAW-001 (SRP) --enforces--> DP-007 (Facade) --achieved_via--> RF-018 (Extract Class)

Claude: "SRP (LAW-001) se conecta con Extract Class (RF-018) en 2 saltos:
1. SRP aplica el Patron Facade (DP-007)
2. Facade a menudo se logra via Extract Class (RF-018)

Insight: Cuando una clase tiene multiples responsabilidades, extraer 
clases (RF-018) ayuda a satisfacer SRP (LAW-001)."
```

---

### 5. `analyze_code`

**Proposito**: Detectar code smells via analisis AST

**Parametros**:
```typescript
{
  code: string
  language: string   // "python", "java", "typescript", etc.
  min_confidence?: number   // Por defecto: 0.5
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

**Ejemplo de conversacion**:
```
Usuario: "Revisa este codigo de validacion de pagos"
[pegar codigo]

Claude llama: analyze_code({
  code: "...",
  language: "python"
})

Retorna:
- SMELL-01: Long Method (confianza 0.87, LOC=45)
- SMELL-08: Long Parameter List (confianza 0.92, params=9)

Claude: "Encontre 2 code smells:

1. Long Method (SMELL-01, confianza 87%)
   - 45 lineas de codigo (umbral: 20)
   - Alta complejidad ciclomatica (12)
   - Recomendado: Extract Method (RF-001)

2. Long Parameter List (SMELL-08, confianza 92%)
   - 9 parametros (umbral: 4)
   - Recomendado: Introduce Parameter Object (RF-029)"
```

---

### 6. `suggest_refactorings`

**Proposito**: Obtener sugerencias de refactoring clasificadas para los smells detectados

**Parametros**:
```typescript
{
  code: string
  language: string
  top_k?: number   // Numero de sugerencias (por defecto: 3)
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

**Ejemplo de conversacion**:
```
Usuario: "¿Que deberia refactorizar en este codigo?"
[pegar clase grande]

Claude llama: suggest_refactorings({
  code: "...",
  language: "java",
  top_k: 3
})

Retorna sugerencias clasificadas para God Object smell:
1. RF-018: Extract Class (prioridad 0.89, severidad Alta, esfuerzo Medio)
2. RF-023: Move Method (prioridad 0.76, severidad Media, esfuerzo Bajo)
3. RF-042: Extract Interface (prioridad 0.65, severidad Baja, esfuerzo Bajo)

Claude: "Este codigo muestra caracteristicas de God Object (SMELL-03).

Prioridades de refactoring:

1. Extract Class (RF-018) - Prioridad 0.89
   - Severidad: Alta
   - Esfuerzo: Medio
   - Mover metodos relacionados con pagos a la clase PaymentProcessor
   - Aplica: SRP (LAW-001), SoC (LAW-012)

2. Move Method (RF-023) - Prioridad 0.76
   - Severidad: Media
   - Esfuerzo: Bajo
   - Reubicar metodos de validacion a la clase Validator
   - Victoria rapida para mejorar cohesion

Comience con #2 (Move Method) para mejora inmediata, 
luego aborde #1 (Extract Class) para diseno a largo plazo."
```

---

## Agentes de IA

Episteme incluye 4 agentes especializados que trabajan juntos como un sistema conectado. Cada agente usa herramientas del grafo de conocimiento para analisis y presenta opciones de seguimiento interactivas que pueden transferirse a otros agentes.

### Red de agentes

```
code-reviewer ←→ episteme-advisor
      ↕                ↕
architecture-analyst ←→ episteme-researcher
      ↕
refactoring-expert (externo)
```

Cada agente finaliza su informe con **Pasos siguientes** — opciones interactivas que conectan con otros agentes o herramientas. Esto crea flujos de trabajo de extremo a extremo desde la deteccion hasta la remediacion.

---

### 1. `code-reviewer`

**Cuando usar**: Revisar codigo en busca de smells, violaciones SOLID y prioridades de refactoring

**Que hace diferente**: Va mas alla de la deteccion — rastrea cadenas de causalidad entre smells (cuales son causas raiz vs. sintomas derivados), valida hallazgos contra convenciones del ecosistema del lenguaje (Rust, Go, etc.) y calibra la severidad contextualmente.

**Herramientas usadas**: `analyze_code` → `suggest_refactorings` → `get_entity` → `get_neighbors` → `find_path`

**Ejemplo de prompt**:
```
"Revisa este modulo de autenticacion"
[pegar codigo]
```

**Respuesta del agente**:
```
# Revision de codigo Episteme

## Resumen
AuthenticationModule tiene un smell God Object como causa raiz que causa 3
problemas derivados. Corregir la causa raiz generara mejoras en cascada.

## Detecciones de smells
| Smell | Ubicacion | Severidad | Confianza | Principio relacionado |
|-------|-----------|-----------|-----------|----------------------|
| God Object (SMELL-03) | auth.rs:1-420 | Critica | confirmado | LAW-001 (SRP) |
| Long Method (SMELL-01) | auth.rs:88-142 | Alta | confirmado | LAW-001 (SRP) |
| Divergent Change (SMELL-09) | auth.rs:200-380 | Media | confirmado | LAW-012 (SoC) |
| Feature Envy (SMELL-15) | auth.rs:290-333 | Media | confirmado | LAW-003 (LSP) |

## Analisis de causalidad
Causas raiz:
- SMELL-03 → causa → SMELL-01, SMELL-09
  (corregir God Object generara mejoras en cascada)

Independiente:
- SMELL-15 — independiente, sin impacto derivado

## Refactorizaciones clasificadas
1. **Extract Class** (RF-018) — Prioridad: Critica
   - Aborda: SMELL-03 (causa raiz)
   - Efecto en cascada: tambien resuelve SMELL-01, SMELL-09
   - Que hacer: Extraer clases SessionManager, EmailVerifier

## Pasos siguientes
1. Refactorizar — Aplicar RF-018 (Extract Class) → usar agente refactoring-expert
2. Profundizar — Explorar cadena de causa raiz para SMELL-03 → usar episteme-advisor
3. Verificacion arquitectonica — Evaluar impacto estructural → usar architecture-analyst
4. Aprender mas — Obtener explicacion completa de SMELL-03 → get_entity(detail_level="full")
```

**Validacion**: Antes de confirmar un smell, el agente verifica contra expresiones idiomaticas del lenguaje (ej: estructuras grandes son normales en Rust), contexto de dominio (archivos de prueba, codigo generado) e impacto real (cosmetico vs. bloqueante para el desarrollo).

---

### 2. `episteme-advisor`

**Cuando usar**: Tomar decisiones de ingenieria (seleccion de patrones, compensaciones arquitectonicas, analisis profundo de causa raiz)

**Herramientas usadas**: `search_knowledge` → `get_entity` → `get_neighbors` → `find_path`

**Ejemplo de prompt**:
```
"¿Deberia usar Singleton para mi clase ConfigManager?"
```

**Respuesta del agente**:
```
# Asesoria: Singleton vs DI para ConfigManager

## Entidades aplicables
### 1. Singleton Pattern (DP-006) - Primario
- **Que dice**: Asegurar que una clase tenga solo una instancia
- **Que significa para usted**: Acceso global a configuracion pero acoplamiento oculto
- **Recomendacion**: Evitar — viola la testeabilidad

### 2. Dependency Injection (DP-018) - Alternativa
- **Que dice**: Inyectar dependencias via constructor
- **Que significa para usted**: Dependencias explicitas, simulacion facil
- **Recomendacion**: Preferir para ConfigManager

## Compromisos
- Singleton (DP-006) ofrece conveniencia pero viola DI (DP-018)
- DI agrega codigo repetitivo pero habilita cumplimiento de LAW-005 (DIP)
- Resolucion: Usar DI con una interfaz de proveedor de configuracion

## Plan de accion
1. Inmediato: Inyectar ConfigManager via constructor
2. Corto plazo: Extraer interfaz ConfigProvider
3. Largo plazo: Considerar estrategias de configuracion especificas por entorno

## Pasos siguientes
1. Implementar — Aplicar patron DI → usar code-reviewer para validar
2. Explorar alternativas — Comparar enfoques → usar episteme-researcher
3. Impacto arquitectonico — Evaluar efectos derivados → usar architecture-analyst
```

---

### 3. `episteme-researcher`

**Cuando usar**: Explorar relaciones en el grafo de conocimiento, encontrar alternativas

**Herramientas usadas**: `search_knowledge` → `get_entity` → `get_neighbors` → `find_path`

**Ejemplo de prompt**:
```
"¿Que leyes aplica el Patron Observer?"
```

**Respuesta del agente**:
```
# Informe de investigacion: Leyes aplicadas por el Patron Observer

## Entidades primarias
1. **Observer Pattern** (DP-012) - Comportamiento
   - Definicion: Definir una dependencia uno a muchos entre objetos
   - Cuando el cambio en un objeto altera otros automaticamente

## Mapa de conexiones
DP-012 --enforces--> LAW-002 (Open/Closed)
DP-012 --enforces--> LAW-005 (Dependency Inversion)
DP-012 --enforces--> LAW-001 (Single Responsibility)

## Contradicciones y compensaciones
- Observer (DP-012) habilita acoplamiento debil pero puede causar cadenas
  de actualizacion inesperadas — contrastado con Mediator (DP-014) que centraliza

## Pasos siguientes
1. Obtener asesoria — Convertir hallazgos en guia → usar episteme-advisor
2. Verificar codigo — Detectar Observer en su base de codigo → usar code-reviewer
3. Evaluar arquitectura — Evaluar ajuste estructural → usar architecture-analyst
```

---

### 4. `architecture-analyst`

**Cuando usar**: Evaluar la arquitectura del sistema contra leyes, patrones y riesgos estructurales

**Herramientas usadas**: `search_knowledge` → `get_entity` → `get_neighbors` → `find_path`

**Ejemplo de prompt**:
```
"¿Esta arquitectura de microservicios sigue la Ley de Conway?"
[describe la arquitectura]
```

**Respuesta del agente**:
```
# Analisis arquitectonico: Microservicios

## Cumplimiento de leyes y principios
| Principio | Estado | Evidencia | Impacto |
|-----------|--------|-----------|---------|
| Conway's Law (LAW-042) | violado | Shipping abarca 2 equipos | Sobrecarga de coordinacion |
| SRP (LAW-001) | en riesgo | Analytics depende de todo | Acoplamiento estricto |
| Bounded Context (LAW-031) | violado | Sin limites de dominio claros | Confusion de datos compartidos |

## Tensiones clave
- Conway's Law (LAW-042) requiere alineacion equipo↔servicio
  pero Shipping service abarca los equipos Commerce + Platform
- Rastreado via: LAW-042 → related_to → LAW-001 → enforced_by → DP-026 (Strangler Fig)

## Recomendaciones arquitectonicas
1. **Critica**: Mover Shipping al equipo Commerce — LAW-042 predice falla de coordinacion
2. **Alta**: Introducir Event Bus para Analytics — desacoplar via eventos asincronos
3. **Media**: Definir Bounded Contexts — alinear limites de servicio con el dominio

## Puntuaciones de cumplimiento
- General: 5/10 | Estructura: 4/10 | Escalabilidad: 6/10 | Mantenibilidad: 5/10

## Pasos siguientes
1. Obtener asesoria — Resolver tensiones clave → usar episteme-advisor
2. Verificar codigo — Detectar smells estructurales → usar code-reviewer
3. Investigar alternativas — Encontrar mejores patrones → usar episteme-researcher
```

---

## Cadenas de flujo de trabajo

Los agentes y herramientas se conectan en pipelines de extremo a extremo. Cada cadena produce un informe seguido de opciones de seguimiento interactivas.

### Cadena 1: Pipeline de revision de codigo
```
analyze_code → suggest_refactorings → get_neighbors("solved_by")
  → find_path(smell_A, smell_B) → Informe con grafo de causalidad
  → El usuario elige: Aplicar correccion / Profundizar / Verificacion arquitectonica / Aprender mas
```

### Cadena 2: Pipeline de revision arquitectonica
```
search_knowledge → get_entity → get_neighbors("enforces")
  → get_neighbors("violates") → find_path → Informe de cumplimiento
  → El usuario elige: Plan de refactoring / Asesoria / Investigar alternativas
```

### Cadena 3: Pipeline de diagnostico de problemas
```
search_knowledge(symptoms) → get_entity → get_neighbors("solved_by")
  → Informe de causa raiz → El usuario elige: Aplicar correccion / Asesoria / Verificar
```

### Cadena 4: Pipeline de aprendizaje
```
search_knowledge(topic) → get_entity → get_neighbors("related_to")
  → Mapa de conceptos → El usuario elige: Ejemplos de codigo / Aplicar al codigo / Comparar
```

### Reglas de encadenamiento entre herramientas

Cada llamada a herramienta lleva naturalmente a la siguiente:

| Despues de llamar... | Siempre hacer seguimiento con... |
|----------------------|----------------------------------|
| `analyze_code` | `suggest_refactorings` sobre los smells detectados |
| `suggest_refactorings` | `get_neighbors(smell_id, "solved_by")` para alternativas |
| `search_knowledge` | `get_entity` sobre los 1-2 resultados principales |
| `get_entity` (smell) | `get_neighbors(id, "violates")` para principios impactados |
| `get_entity` (patron) | `get_neighbors(id, "enforces")` para leyes aplicadas |
| Multiples smells detectados | `find_path(smell_A, smell_B)` para mapeo de causalidad |

---

## Instalacion para otras herramientas

### Cursor

```bash
epis install cursor
```

Agrega la configuracion MCP a `~/.cursor/mcp.json`:
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

Genera `AGENTS.md` en la raiz del proyecto con definiciones de agentes.

### Integracion MCP personalizada

Si su herramienta soporta MCP, configure manualmente:

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

## Ejecucion como servicio en segundo plano

Para mejor rendimiento, ejecute Episteme MCP como un proxy HTTP persistente:

```bash
# Iniciar servicio en segundo plano
epis service start

# Verificar estado
epis service status
# Salida: Running on http://localhost:43175 (PID 12345)

# Habilitar inicio automatico al arrancar (macOS)
epis service enable

# Detener servicio
epis service stop
```

Actualice la configuracion MCP para usar el proxy HTTP:

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

## Solucion de problemas

### Las herramientas no aparecen en Claude

1. Verifique que el archivo de configuracion existe: `cat ~/.claude/claude_desktop_config.json`
2. Verifique que episteme esta en el PATH: `which episteme`
3. Pruebe MCP directamente: `episteme mcp`
4. Revise los registros: `tail -f ~/.episteme/logs/mcp.err.log`

### Error "Database not found"

```bash
# Reconstruir la base de datos de conocimiento
epis build --rebuild
```

### Respuestas de busqueda lentas

```bash
# Usar aceleracion GPU
epis build --gpu

# O ejecutar como servicio en segundo plano (calentamiento mas rapido)
epis service start
```

### El agente no usa las herramientas

Asegurese de que el agente tenga capacidad de llamada a herramientas. En Claude Code:
```
Usuario: "Usa Episteme para encontrar patrones para logica de reintentos"
      ^^^^ mencionar explicitamente el uso de herramientas
```

---

## Avanzado: Integracion de conocimiento personalizado

Combine Episteme (conocimiento generico) con Alcove (conocimiento de equipo):

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

Consulte la [Guia de integracion Alcove](./alcove-integration.md) para patrones de fuente dual.

---

## Alternativa API

Si su herramienta de IA no soporta MCP, use la API REST:

```bash
# Iniciar servidor API
docker-compose up -d

# Usar desde cualquier herramienta
curl http://localhost:8000/search?q=strategy+pattern
```

Consulte la [Documentacion de API](./api.md) para los endpoints.

---

## Activacion automatica (Claude Code)

Cuando describe un problema en lenguaje natural, Claude Code detecta automaticamente la intencion y llama a la herramienta Episteme apropiada — **no necesita mencionar Episteme explicitamente**. A continuacion se muestran los patrones de activacion exactos y ejemplos.

### Como funciona

```
Su entrada en lenguaje natural
    ↓ Claude detecta palabras clave/patrones
    ↓ La herramienta Episteme se llama automaticamente
    ↓ El grafo de conocimiento retorna datos verificados
    ↓ (Patrones de diseno · Code Smells · Tecnicas de Refactoring · Leyes de Ingenieria)
    ↓ La respuesta de Claude esta fundamentada en evidencia
```

> **Nota:** Esta es una activacion automatica basada en prompts, no un gancho estricto. Para garantizar una llamada, use el skill `/episteme` directamente.

### Problemas de estructura de codigo

| Lo que dice (ejemplos) | Lo que Episteme detecta | Llamada a herramienta automatica |
|------------------------|------------------------|--------------------------------|
| "Esta clase hace demasiado", "Este archivo tiene mas de 300 lineas" | God Class, Large Class, Single Responsibility | `search_knowledge("god class large class single responsibility")` |
| "Esta funcion es muy larga", "Demasiadas lineas en este metodo" | Long Method | `search_knowledge("long method extract method")` |
| "El codigo es muy complejo", "Dificil de seguir" | Complexity, Cognitive Overload | `search_knowledge("complexity smell cognitive overload")` |
| "Copie y pegue esto en todas partes", "Hay logica duplicada" | Duplicated Code, Clone | `search_knowledge("duplicated code clone smell")` |

### Problemas de acoplamiento y dependencias

| Lo que dice (ejemplos) | Lo que Episteme detecta | Llamada a herramienta automatica |
|------------------------|------------------------|--------------------------------|
| "La logica de negocio llama a la DB directamente" | Coupling, Persistence, Repository | `search_knowledge("coupling persistence repository data access layer")` |
| "Cambiar X rompe Y", "Los cambios se propagan a todas partes" | Brittle Coupling, Change Propagation | `search_knowledge("brittle coupling change propagation rigidity")` |
| "Agregar un nuevo tipo significa tocar todo", "switch-case sigue creciendo" | Open/Closed, Strategy, Polymorphism | `search_knowledge("open closed principle strategy polymorphism")` |

### Problemas de testing y calidad

| Lo que dice (ejemplos) | Lo que Episteme detecta | Llamada a herramienta automatica |
|------------------------|------------------------|--------------------------------|
| "Esto es dificil de probar", "No puedo escribir tests unitarios para esto" | Testability, Dependency Injection | `search_knowledge("testability dependency injection mockability")` |

### Problemas de rendimiento y concurrencia

| Lo que dice (ejemplos) | Lo que Episteme detecta | Llamada a herramienta automatica |
|------------------------|------------------------|--------------------------------|
| "La API es lenta", "El tiempo de respuesta es muy alto" | N+1 Query, Lazy Loading, Caching | `search_knowledge("N+1 query lazy loading caching performance")` |
| "¿Es esto thread-safe?", "¿Hay problemas de concurrencia?" | Thread Safety, Race Condition | `search_knowledge("thread safety race condition shared mutable state")` |

### Revision y analisis de codigo

| Lo que dice (ejemplos) | Llamada a herramienta automatica |
|------------------------|--------------------------------|
| Comparte un fragmento de codigo y pide revision | `analyze_code(code)` → `suggest_refactorings(code)` |
| "Mejora este codigo", "Refactoriza esto por mi" | `suggest_refactorings(code)` |

### Discusiones de arquitectura

| Lo que dice (ejemplos) | Llamada a herramienta automatica |
|------------------------|--------------------------------|
| "Microservicios vs monolito", "¿Como deberiamos dividir esto?" | `search_knowledge("monolith microservice decomposition bounded context")` |
| "¿Esta bien esta arquitectura?", "Revisa este diseno" | `search_knowledge("layered architecture coupling responsibility")` |

### Ejemplos de conversaciones del mundo real

**Ejemplo 1 — Detectando un God Object:**

```
Usted: "UserService esta saliendo de control. Maneja autenticacion,
        gestion de sesiones, verificacion de email y restablecimiento de contrasenas."

Claude (auto-llama search_knowledge):
  → Detecta: God Object / violacion de Single Responsibility
  → Retorna: SMELL-03 (God Object), LAW-001 (SRP), RF-018 (Extract Class)

Claude: "UserService muestra caracteristicas de God Object (SMELL-03).
  Viola el Principio de Responsabilidad Unica (LAW-001).

  Refactorizaciones recomendadas:
  1. Extract Class (RF-018) — Crear SessionManager para logica de sesiones
  2. Move Method (RF-023) — Mover verificacion de email a EmailService
  3. Extract Interface (RF-042) — Definir interfaz AuthService

  Comience con #2 para victorias rapidas, luego #1 para mejora estructural."
```

**Ejemplo 2 — Investigacion de rendimiento:**

```
Usted: "La API de listado de productos es muy lenta. Carga bien individualmente
        pero la pagina de lista tarda mas de 5 segundos."

Claude (auto-llama search_knowledge):
  → Detecta: Problema N+1 Query
  → Retorna: Patrones de rendimiento, Lazy Loading, estrategias de Caching

Claude: "Esto suena como un problema N+1 query — cada producto activa
  llamadas DB individuales en lugar de una consulta por lotes.

  Soluciones del grafo de conocimiento:
  1. Batch loading / Eager loading — Obtener datos relacionados en una consulta
  2. Data Mapper (DP-019) — Desacoplar logica de consulta de objetos de dominio
  3. Aplicar Lazy Loading (RF-050) — Cargar relaciones solo bajo demanda

  ¿Le gustaria que verifique el codigo para patrones N+1?"
```

**Ejemplo 3 — Revision de codigo con fragmento:**

```
Usted: [pegar codigo Python]
       "¿Puedes revisar esto? Algo no se siente bien."

Claude (auto-llama analyze_code → suggest_refactorings):
  → analyze_code detecta: Long Method (confianza 0.87), Long Parameter List (0.92)
  → suggest_refactorings retorna sugerencias clasificadas

Claude: "Encontre 2 code smells:

  1. Long Method (SMELL-01) — 45 lineas, complejidad ciclomatica 12
     → Extract Method (RF-001), prioridad 0.89

  2. Long Parameter List (SMELL-08) — 9 parametros
     → Introduce Parameter Object (RF-029), prioridad 0.92

  Comience con RF-029 (Parameter Object) — es la prioridad mas alta
  y facilita el posterior Extract Method."
```

---

## Pasos siguientes

1. **Probar agentes**: Pregunte a episteme-advisor "¿Deberia usar Singleton?"
2. **Analizar codigo**: Pegue una funcion y pida a code-reviewer que verifique smells
3. **Explorar el grafo**: Use episteme-researcher para encontrar relaciones entre patrones
4. **Flujos de trabajo personalizados**: Combine herramientas (analyze → suggest → search)

Para mas ejemplos, consulte:
- [Integracion Alcove](./alcove-integration.md) — Conocimiento de equipo + Episteme
- [Configuracion de monitoreo](../../monitoring/README.md) — Rastrear uso de patrones
- [Referencia API](./api.md) — Endpoints REST
