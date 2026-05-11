# MCP-Integrationsleitfaden

> Integrieren Sie Epistemes Wissensgraph in Claude Code, Cursor und andere MCP-kompatible KI-Tools

## Rust MCP-HTTP-Modus (Aktuell)
Verwenden Sie den eigenständigen HTTP-Transport direkt:

```bash
# MCP über HTTP starten
episteme mcp --http --host 127.0.0.1 --port 43175
```

Authentifizierungsverhalten:
- Wenn `EPISTEME_API_KEYS` konfiguriert ist, müssen Anfragen Folgendes enthalten:
```http
Authorization: Bearer <api-key>
```
- Wenn keine Schlüssel konfiguriert sind, wird die Authentifizierung übersprungen (Entwicklungsmodus).
- `GET /health` ist für Health-Checks immer öffentlich zugänglich.

Hinweis:
- `epis service` verwaltet denselben MCP-HTTP-Modus im Hintergrund (`start|stop|status|enable|disable`).
- Ältere `--proxy`-Beispiele sind veraltet; verwenden Sie `mcp --http`/`service` direkt.

## Was ist MCP?

Das [Model Context Protocol (MCP)](https://modelcontextprotocol.io) ist ein offener Standard, der KI-Assistenten den Zugriff auf externe Tools und Datenquellen ermöglicht. Episteme stellt 6 MCP-Tools bereit, die KI-Agenten direkten Zugriff auf Software-Engineering-Wissen geben.

---

## Schnellstart (Claude Code)

### 1. Episteme installieren

```bash
# Installation (erfordert Rust 1.95+)
cargo install --git https://github.com/epicsagas/Episteme

# Agenten und MCP-Server in Claude Code installieren
# (lädt Daten herunter und konfiguriert MCP automatisch)
epis install claude
```

> Wenn der Daten-Download fehlschlägt, verwenden Sie die Quellcode-Installation: `git clone` → `cargo build --release` → `epis install --local`

### 2. Installation überprüfen

Prüfen Sie `~/.claude/claude_desktop_config.json`:

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

### 3. Loslegen

Starten Sie Claude Code neu. Sie haben nun Zugriff auf 6 Episteme-Tools:

```
Benutzer: "Was ist der beste Weg, um einen God Object-Smell zu beheben?"

Claude (verwendet das search_knowledge-Tool):
  → Sucht nach "God Object"-Refactorings
  → Gibt zurück: RF-018 (Extract Class), RF-023 (Move Method)

Claude: "Das God Object-Anti-Pattern (SMELL-03) verletzt das Single
Responsibility Principle (LAW-001). Beste Refactorings:

1. Extract Class (RF-018) - Verwandte Methoden/Felder in neue Klasse verschieben
2. Move Method (RF-023) - Methoden in appropriate Klassen umsiedeln

Beide erzwingen SOLID-Prinzipien und verbessern die Testbarkeit."
```

---

## MCP-Tools-Referenz

### 1. `search_knowledge`

**Zweck**: Semantische Suche über alle Entitäten (Patterns, Gesetze, Refactorings, Smells)

**Parameter**:
```typescript
{
  query: string          // Natürlichsprachliche Anfrage
  top_k?: number         // Anzahl der Ergebnisse (Standard: 5)
  filter_type?: string   // "pattern", "law", "refactoring", "smell"
}
```

**Rückgabe**:
```typescript
{
  results: [{
    entity_id: string     // z. B. "DP-023"
    title: string
    entity_type: string
    similarity: number    // 0.0-1.0
    summary: string
  }]
}
```

**Beispielkonversation**:
```
Benutzer: "Wie kann ich meinen Code testbarer machen?"

Claude ruft auf: search_knowledge({
  query: "improve testability",
  top_k: 3
})

Rückgabe:
- LAW-001: Single Responsibility Principle
- DP-018: Dependency Injection
- RF-042: Extract Interface

Claude: "Drei zentrale Ansätze zur Verbesserung der Testbarkeit:
1. SRP anwenden (LAW-001) - Eine Klasse, ein Änderungsgrund
2. Dependency Injection verwenden (DP-023) - Abhängigkeiten injizieren
3. Interface extrahieren (RF-042) - Externe Abhängigkeiten mocken"
```

---

### 2. `get_entity`

**Zweck**: Vollständige Details einer bestimmten Entität anhand ihrer ID abrufen

**Parameter**:
```typescript
{
  entity_id: string   // z. B. "DP-023", "RF-001", "SMELL-01"
}
```

**Rückgabe**:
```typescript
{
  entity_id: string
  title: string
  type: string
  description: string
  implementation: string    // Code-Beispiele
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

**Beispielkonversation**:
```
Benutzer: "Erklären Sie das Strategy Pattern im Detail"

Claude ruft auf: get_entity({ entity_id: "DP-023" })

Rückgabe:
- Vollständige Beschreibung des Strategy Patterns
- Wann zu verwenden (mehrere Algorithmen, Laufzeit-Umschaltung)
- Implementierung (Context, Strategy-Interface, ConcreteStrategies)
- Vorteile: Open/Closed Principle, Laufzeit-Flexibilität
- Nachteile: Erhöhte Klassenzahl
- Verwandt: LAW-002 (Open/Closed), RF-034 (Replace Conditional with Polymorphism)

Claude: "Das Strategy Pattern (DP-023) kapselt Algorithmen in separaten Klassen...
[liefert detaillierte Erklärung mit Code-Beispiel]

Verwandte Konzepte:
- Erzwingt das Open/Closed Principle (LAW-002)
- Oft angewendet via Replace Conditional with Polymorphism (RF-034)"
```

---

### 3. `get_neighbors`

**Zweck**: Mit einer Entität verbundene Entitäten erkunden

**Parameter**:
```typescript
{
  entity_id: string
  relation_type?: string   // "solves", "enforces", "violates", "relates_to"
  max_depth?: number       // Standard: 1
}
```

**Rückgabe**:
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

**Beispielkonversation**:
```
Benutzer: "Welche Refactorings lösen den Long Method-Smell?"

Claude ruft auf: get_neighbors({
  entity_id: "SMELL-01",
  relation_type: "solved_by"
})

Rückgabe:
- RF-001: Extract Method
- RF-034: Decompose Conditional
- RF-056: Replace Temp with Query

Claude: "Long Method (SMELL-01) kann durch Folgendes gelöst werden:
1. Extract Method (RF-001) - In kleinere Methoden aufbrechen
2. Decompose Conditional (RF-034) - Komplexe Bedingungen vereinfachen
3. Replace Temp with Query (RF-056) - Temporäre Variablen eliminieren"
```

---

### 4. `find_path`

**Zweck**: Verbindung zwischen zwei Entitäten finden (z. B. wie ein Gesetz mit einem Pattern zusammenhängt)

**Parameter**:
```typescript
{
  from_id: string
  to_id: string
  max_depth?: number   // Standard: 5
}
```

**Rückgabe**:
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

**Beispielkonversation**:
```
Benutzer: "Wie hängt das Single Responsibility Principle mit dem Extract Class-Refactoring zusammen?"

Claude ruft auf: find_path({
  from_id: "LAW-001",
  to_id: "RF-018"
})

Rückgabe-Pfad:
LAW-001 (SRP) --enforces--> DP-007 (Facade) --achieved_via--> RF-018 (Extract Class)

Claude: "SRP (LAW-001) verbindet sich in 2 Hops mit Extract Class (RF-018):
1. SRP erzwingt das Facade Pattern (DP-007)
2. Facade wird oft via Extract Class (RF-018) erreicht

Erkenntnis: Wenn eine Klasse mehrere Verantwortlichkeiten hat, hilft
das Extrahieren von Klassen (RF-018), das SRP (LAW-001) zu erfüllen."
```

---

### 5. `analyze_code`

**Zweck**: Code-Smells mittels AST-Analyse erkennen

**Parameter**:
```typescript
{
  code: string
  language: string   // "python", "java", "typescript" usw.
  min_confidence?: number   // Standard: 0.5
}
```

**Rückgabe**:
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

**Beispielkonversation**:
```
Benutzer: "Überprüfen Sie diesen Zahlungsvalidierungscode"
[fügt Code ein]

Claude ruft auf: analyze_code({
  code: "...",
  language: "python"
})

Rückgabe:
- SMELL-01: Long Method (Konfidenz 0.87, LOC=45)
- SMELL-08: Long Parameter List (Konfidenz 0.92, Parameter=9)

Claude: "Ich habe 2 Code-Smells gefunden:

1. Long Method (SMELL-01, Konfidenz 87%)
   - 45 Codezeilen (Schwellenwert: 20)
   - Hohe zyklomatische Komplexität (12)
   - Empfehlung: Extract Method (RF-001)

2. Long Parameter List (SMELL-08, Konfidenz 92%)
   - 9 Parameter (Schwellenwert: 4)
   - Empfehlung: Introduce Parameter Object (RF-029)"
```

---

### 6. `suggest_refactorings`

**Zweck**: Priorisierte Refactoring-Vorschläge für erkannte Smells erhalten

**Parameter**:
```typescript
{
  code: string
  language: string
  top_k?: number   // Anzahl der Vorschläge (Standard: 3)
  min_confidence?: number
}
```

**Rückgabe**:
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

**Beispielkonversation**:
```
Benutzer: "Was sollte ich in diesem Code refactoren?"
[fügt große Klasse ein]

Claude ruft auf: suggest_refactorings({
  code: "...",
  language: "java",
  top_k: 3
})

Rückgabe priorisierte Vorschläge für God Object-Smell:
1. RF-018: Extract Class (Priorität 0.89, Hoher Schweregrad, Mittlerer Aufwand)
2. RF-023: Move Method (Priorität 0.76, Mittlerer Schweregrad, Niedriger Aufwand)
3. RF-042: Extract Interface (Priorität 0.65, Niedriger Schweregrad, Niedriger Aufwand)

Claude: "Dieser Code zeigt God Object-Eigenschaften (SMELL-03).

Top-Refactoring-Prioritäten:

1. Extract Class (RF-018) - Priorität 0.89
   - Schweregrad: Hoch
   - Aufwand: Mittel
   - Zahlungsbezogene Methoden in PaymentProcessor-Klasse verschieben
   - Erzwingt: SRP (LAW-001), SoC (LAW-012)

2. Move Method (RF-023) - Priorität 0.76
   - Schweregrad: Mittel
   - Aufwand: Niedrig
   - Validierungsmethoden in Validator-Klasse umsiedeln
   - Schneller Erfolg für besseren Zusammenhalt

Beginnen Sie mit #2 (Move Method) für sofortige Verbesserung,
danach #1 (Extract Class) für langfristiges Design."
```

---

## KI-Agenten

Episteme umfasst 4 spezialisierte Agenten, die als verbundenes System zusammenarbeiten. Jeder Agent verwendet Wissensgraph-Tools für Analysen und bietet interaktive Folgeschritte, die an andere Agenten übergeben werden können.

### Agenten-Netzwerk

```
code-reviewer ←→ episteme-advisor
      ↕                ↕
architecture-analyst ←→ episteme-researcher
      ↕
refactoring-expert (extern)
```

Jeder Agent schließt seinen Bericht mit **Nächste Schritte** ab — interaktive Optionen, die mit anderen Agenten oder Tools verknüpft sind. Dies ermöglicht End-to-End-Workflows von der Erkennung bis zur Behebung.

---

### 1. `code-reviewer`

**Verwendung**: Code-Überprüfung auf Smells, SOLID-Verstöße und Refactoring-Prioritäten

**Besonderheit**: Geht über die Erkennung hinaus — verfolgt Kausalitätsketten zwischen Smells (welche Smells sind Ursachen vs. nachgelagerte Symptome), validiert Ergebnisse anhand von Sprachökosystem-Konventionen (Rust, Go usw.) und kalibriert den Schweregrad kontextbezogen.

**Verwendete Tools**: `analyze_code` → `suggest_refactorings` → `get_entity` → `get_neighbors` → `find_path`

**Beispiel-Prompt**:
```
"Überprüfen Sie dieses Authentifizierungsmodul"
[fügt Code ein]
```

**Agentenantwort**:
```
# Episteme Code-Review

## Zusammenfassung
AuthenticationModule hat einen ursächlichen God Object-Smell, der
3 nachgelagerte Probleme verursacht. Die Behebung der Ursache führt
zu kaskadenartigen Verbesserungen.

## Smell-Erkennungen
| Smell | Ort | Schweregrad | Konfidenz | Verwandtes Prinzip |
|-------|-----|-------------|-----------|-------------------|
| God Object (SMELL-03) | auth.rs:1-420 | Kritisch | bestätigt | LAW-001 (SRP) |
| Long Method (SMELL-01) | auth.rs:88-142 | Hoch | bestätigt | LAW-001 (SRP) |
| Divergent Change (SMELL-09) | auth.rs:200-380 | Mittel | bestätigt | LAW-012 (SoC) |
| Feature Envy (SMELL-15) | auth.rs:290-333 | Mittel | bestätigt | LAW-003 (LSP) |

## Kausalitätsanalyse
Ursachen:
- SMELL-03 → verursacht → SMELL-01, SMELL-09
  (Behebung von God Object führt zu Kaskadenverbesserung)

Unabhängig:
- SMELL-15 — eigenständig, keine nachgelagerten Auswirkungen

## Priorisierte Refactorings
1. **Extract Class** (RF-018) — Priorität: Kritisch
   - Behebt: SMELL-03 (Ursache)
   - Kaskadeneffekt: löst auch SMELL-01, SMELL-09
   - Vorgehen: SessionManager-, EmailVerifier-Klassen extrahieren

## Nächste Schritte
1. Refactoren — RF-018 (Extract Class) anwenden → refactoring-expert-Agent verwenden
2. Vertiefung — Ursachenkette für SMELL-03 erkunden → episteme-advisor verwenden
3. Architektur-Check — Strukturelle Auswirkungen bewerten → architecture-analyst verwenden
4. Mehr erfahren — Vollständige Erklärung von SMELL-03 → get_entity(detail_level="full")
```

**Validierung**: Vor der Bestätigung eines Smells gleicht der Agent mit Sprach-Idiomen ab (z. B. große Structs sind in Rust normal), dem Domänenkontext (Testdateien, generierter Code) und der tatsächlichen Auswirkung (kosmetisch vs. entwicklungsblockierend).

---

### 2. `episteme-advisor`

**Verwendung**: Engineering-Entscheidungen treffen (Pattern-Auswahl, Architektur-Trade-offs, Ursachenvertiefung)

**Verwendete Tools**: `search_knowledge` → `get_entity` → `get_neighbors` → `find_path`

**Beispiel-Prompt**:
```
"Sollte ich Singleton für meine ConfigManager-Klasse verwenden?"
```

**Agentenantwort**:
```
# Beratung: Singleton vs. DI für ConfigManager

## Anwendbare Entitäten
### 1. Singleton Pattern (DP-006) - Primär
- **Aussage**: Sicherstellen, dass eine Klasse nur eine Instanz hat
- **Bedeutung für Sie**: Globaler Konfigurationszugriff, aber versteckte Kopplung
- **Empfehlung**: Vermeiden — verletzt Testbarkeit

### 2. Dependency Injection (DP-018) - Alternative
- **Aussage**: Abhängigkeiten über Konstruktor injizieren
- **Bedeutung für Sie**: Explizite Abhängigkeiten, einfaches Mocking
- **Empfehlung**: Für ConfigManager bevorzugen

## Trade-offs
- Singleton (DP-006) bietet Bequemlichkeit, verletzt aber DI (DP-018)
- DI fügt Boilerplate hinzu, ermöglicht aber LAW-005 (DIP)-Compliance
- Lösung: DI mit ConfigProvider-Interface verwenden

## Aktionsplan
1. Sofort: ConfigManager über Konstruktor injizieren
2. Kurzfristig: ConfigProvider-Interface extrahieren
3. Langfristig: Umgebungsabhängige Konfigurationsstrategien in Betracht ziehen

## Nächste Schritte
1. Implementieren — DI-Pattern anwenden → mit code-reviewer validieren
2. Alternativen erkunden — Ansätze vergleichen → episteme-researcher verwenden
3. Architektur-Auswirkung — Nachgelagerte Effekte bewerten → architecture-analyst verwenden
```

---

### 3. `episteme-researcher`

**Verwendung**: Beziehungen im Wissensgraph erkunden, Alternativen finden

**Verwendete Tools**: `search_knowledge` → `get_entity` → `get_neighbors` → `find_path`

**Beispiel-Prompt**:
```
"Welche Gesetze erzwingt das Observer Pattern?"
```

**Agentenantwort**:
```
# Forschungsbericht: Vom Observer Pattern erzwingte Gesetze

## Primäre Entitäten
1. **Observer Pattern** (DP-012) - Behavioral
   - Definition: Eine-zu-viele-Abhängigkeit zwischen Objekten definieren
   - Wenn Änderungen an einem Objekt automatisch andere beeinflussen

## Verbindungskarte
DP-012 --enforces--> LAW-002 (Open/Closed)
DP-012 --enforces--> LAW-005 (Dependency Inversion)
DP-012 --enforces--> LAW-001 (Single Responsibility)

## Widersprüche & Trade-offs
- Observer (DP-012) ermöglicht lose Kopplung, kann aber unerwartete
  Update-Ketten verursachen — im Kontrast zu Mediator (DP-014), der zentralisiert

## Nächste Schritte
1. Beratung einholen — Ergebnisse in Handlungsanleitung umwandeln → episteme-advisor verwenden
2. Code prüfen — Observer in Ihrer Codebasis erkennen → code-reviewer verwenden
3. Architektur bewerten — Strukturelle Eignung beurteilen → architecture-analyst verwenden
```

---

### 4. `architecture-analyst`

**Verwendung**: Systemarchitektur anhand von Gesetzen, Patterns und strukturellen Risiken bewerten

**Verwendete Tools**: `search_knowledge` → `get_entity` → `get_neighbors` → `find_path`

**Beispiel-Prompt**:
```
"Folgt diese Microservices-Archuktur Conway's Law?"
[beschreibt Architektur]
```

**Agentenantwort**:
```
# Architektur-Analyse: Microservices

## Gesetz- & Prinzipien-Compliance
| Prinzip | Status | Nachweis | Auswirkung |
|---------|--------|-----------|------------|
| Conway's Law (LAW-042) | verletzt | Shipping erstreckt sich über 2 Teams | Koordinationsaufwand |
| SRP (LAW-001) | gefährdet | Analytics hängt von allem ab | Enge Kopplung |
| Bounded Context (LAW-031) | verletzt | Keine klaren Domänengrenzen | Gemeinsame Datenverwirrung |

## Zentrale Spannungen
- Conway's Law (LAW-042) erfordert Team↔Service-Ausrichtung,
  aber Shipping-Service erstreckt sich über Commerce + Platform-Teams
- Rückverfolgt via: LAW-042 → related_to → LAW-001 → enforced_by → DP-026 (Strangler Fig)

## Architektur-Empfehlungen
1. **Kritisch**: Shipping zum Commerce-Team verschieben — LAW-042 sagt Koordinationsversagen voraus
2. **Hoch**: Event Bus für Analytics einführen — über asynchrone Events entkoppeln
3. **Mittel**: Bounded Contexts definieren — Service-Grenzen an Domäne ausrichten

## Compliance-Scores
- Gesamt: 5/10 | Struktur: 4/10 | Skalierbarkeit: 6/10 | Wartbarkeit: 5/10

## Nächste Schritte
1. Beratung einholen — Zentrale Spannungen auflösen → episteme-advisor verwenden
2. Code prüfen — Strukturelle Smells erkennen → code-reviewer verwenden
3. Alternativen erforschen — Bessere Patterns finden → episteme-researcher verwenden
```

---

## Workflow-Ketten

Agenten und Tools verbinden sich zu End-to-End-Pipelines. Jede Kette erzeugt einen Bericht gefolgt von interaktiven Folgeschritten.

### Kette 1: Code-Review-Pipeline
```
analyze_code → suggest_refactorings → get_neighbors("solved_by")
  → find_path(smell_A, smell_B) → Bericht mit Kausalitätsgraph
  → Benutzer wählt: Fix anwenden / Vertiefen / Architektur-Check / Mehr erfahren
```

### Kette 2: Architektur-Review-Pipeline
```
search_knowledge → get_entity → get_neighbors("enforces")
  → get_neighbors("violates") → find_path → Compliance-Bericht
  → Benutzer wählt: Refactoring-Plan / Beratung / Alternativen erforschen
```

### Kette 3: Problemdiagnose-Pipeline
```
search_knowledge(Symptome) → get_entity → get_neighbors("solved_by")
  → Ursachenbericht → Benutzer wählt: Fix anwenden / Beratung / Verifizieren
```

### Kette 4: Lern-Pipeline
```
search_knowledge(Thema) → get_entity → get_neighbors("related_to")
  → Konzeptkarte → Benutzer wählt: Code-Beispiele / Auf Code anwenden / Vergleichen
```

### Tool-übergreifende Verkettungsregeln

Jeder Tool-Aufruf führt natürlicherweise zum nächsten:

| Nach Aufruf von... | Immer folgen mit... |
|---------------------|---------------------|
| `analyze_code` | `suggest_refactorings` für erkannte Smells |
| `suggest_refactorings` | `get_neighbors(smell_id, "solved_by")` für Alternativen |
| `search_knowledge` | `get_entity` für die Top 1-2 Ergebnisse |
| `get_entity` (Smell) | `get_neighbors(id, "violates")` für betroffene Prinzipien |
| `get_entity` (Pattern) | `get_neighbors(id, "enforces")` für erzwingende Gesetze |
| Mehrere Smells erkannt | `find_path(smell_A, smell_B)` für Kausalitätsmapping |

---

## Installation für andere Tools

### Cursor

```bash
epis install cursor
```

Fügt MCP-Konfiguration zu `~/.cursor/mcp.json` hinzu:
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

Erzeugt `AGENTS.md` im Projektverzeichnis mit Agenten-Definitionen.

### Benutzerdefinierte MCP-Integration

Wenn Ihr Tool MCP unterstützt, konfigurieren Sie manuell:

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

## Als Hintergrunddienst ausführen

Für bessere Leistung führen Sie Episteme MCP als persistenten HTTP-Proxy aus:

```bash
# Hintergrunddienst starten
epis service start

# Status prüfen
epis service status
# Ausgabe: Running on http://localhost:43175 (PID 12345)

# Auto-Start beim Boot aktivieren (macOS)
epis service enable

# Dienst stoppen
epis service stop
```

MCP-Konfiguration für HTTP-Proxy aktualisieren:

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

Logs: `~/.episteme/logs/mcp.out.log`

---

## Fehlerbehebung

### Tools werden in Claude nicht angezeigt

1. Konfigurationsdatei vorhanden prüfen: `cat ~/.claude/claude_desktop_config.json`
2. Episteme im PATH verifizieren: `which episteme`
3. MCP direkt testen: `episteme mcp`
4. Logs prüfen: `tail -f ~/.episteme/logs/mcp.err.log`

### "Database not found"-Fehler

```bash
# Wissensdatenbank neu erstellen
epis build --rebuild
```

### Langsame Suchantworten

```bash
# GPU-Beschleunigung verwenden
epis build --gpu

# Oder als Hintergrunddienst ausführen (schnellerer Warmup)
epis service start
```

### Agent nutzt Tools nicht

Stellen Sie sicher, dass der Agent Tool-Aufrufe unterstützt. In Claude Code:
```
Benutzer: "Verwenden Sie Episteme, um Patterns für Retry-Logik zu finden"
      ^^^^ Tool-Verwendung explizit erwähnen
```

---

## Fortgeschritten: Benutzerdefinierte Wissensintegration

Episteme (allgemeines Wissen) mit Alcove (Team-Wissen) kombinieren:

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

Siehe [Alcove-Integrationsleitfaden](./alcove-integration.md) für Dual-Source-Patterns.

---

## API-Alternative

Wenn Ihr KI-Tool MCP nicht unterstützt, verwenden Sie die REST-API:

```bash
# API-Server starten
docker-compose up -d

# Von jedem Tool aus verwenden
curl http://localhost:8000/search?q=strategy+pattern
```

Siehe [API-Dokumentation](./api.md) für Endpunkte.

---

## Automatische Auslösung (Claude Code)

Wenn Sie ein Problem in natürlicher Sprache beschreiben, erkennt Claude Code automatisch die Absicht und ruft das entsprechende Episteme-Tool auf — **Sie müssen Episteme nicht explizit erwähnen**. Nachfolgend die genauen Trigger-Patterns und Beispiele.

### Funktionsweise

```
Ihre natürlichsprachliche Eingabe
    ↓ Claude erkennt Schlüsselwörter/Patterns
    ↓ Episteme-Tool wird automatisch aufgerufen
    ↓ Wissensgraph liefert verifizierte Daten
    ↓ (Design Patterns · Code Smells · Refactoring-Techniken · Engineering-Gesetze)
    ↓ Claudes Antwort ist evidenzbasiert
```

> **Hinweis:** Dies ist prompt-basiertes Auto-Triggering, kein harter Hook. Um einen Aufruf zu garantieren, verwenden Sie direkt den `/episteme`-Skill.

### Code-Strukturprobleme

| Was Sie sagen (Beispiele) | Was Episteme erkennt | Automatischer Tool-Aufruf |
|--------------------------|---------------------|--------------------------|
| "Diese Klasse macht zu viel", "Diese Datei hat über 300 Zeilen" | God Class, Large Class, Single Responsibility | `search_knowledge("god class large class single responsibility")` |
| "Diese Funktion ist zu lang", "Zu viele Zeilen in dieser Methode" | Long Method | `search_knowledge("long method extract method")` |
| "Der Code ist zu komplex", "Schwer zu verstehen" | Komplexität, Cognitive Overload | `search_knowledge("complexity smell cognitive overload")` |
| "Ich habe das überall hin kopiert", "Es gibt duplizierte Logik" | Duplicate Code, Clone | `search_knowledge("duplicated code clone smell")` |

### Kopplungs- und Abhängigkeitsprobleme

| Was Sie sagen (Beispiele) | Was Episteme erkennt | Automatischer Tool-Aufruf |
|--------------------------|---------------------|--------------------------|
| "Geschäftslogik ruft DB direkt auf" | Kopplung, Persistenz, Repository | `search_knowledge("coupling persistence repository data access layer")` |
| "X ändern bricht Y", "Änderungen pflanzen sich überall fort" | Brittle Coupling, Change Propagation | `search_knowledge("brittle coupling change propagation rigidity")` |
| "Einen neuen Typ hinzufügen bedeutet alles anfassen", "switch-case wächst ständig" | Open/Closed, Strategy, Polymorphism | `search_knowledge("open closed principle strategy polymorphism")` |

### Test- und Qualitätsprobleme

| Was Sie sagen (Beispiele) | Was Episteme erkennt | Automatischer Tool-Aufruf |
|--------------------------|---------------------|--------------------------|
| "Das ist schwer zu testen", "Keine Unit-Tests dafür möglich" | Testbarkeit, Dependency Injection | `search_knowledge("testability dependency injection mockability")` |

### Performance- und Nebenläufigkeitsprobleme

| Was Sie sagen (Beispiele) | Was Episteme erkennt | Automatischer Tool-Aufruf |
|--------------------------|---------------------|--------------------------|
| "Die API ist langsam", "Antwortzeit ist zu hoch" | N+1 Query, Lazy Loading, Caching | `search_knowledge("N+1 query lazy loading caching performance")` |
| "Ist das Thread-sicher?", "Nebenläufigkeitsprobleme?" | Thread Safety, Race Condition | `search_knowledge("thread safety race condition shared mutable state")` |

### Code-Review und Analyse

| Was Sie sagen (Beispiele) | Automatischer Tool-Aufruf |
|--------------------------|--------------------------|
| Code-Snippet teilen und um Review bitten | `analyze_code(code)` → `suggest_refactorings(code)` |
| "Verbessern Sie diesen Code", "Refactoren Sie das für mich" | `suggest_refactorings(code)` |

### Architektur-Diskussionen

| Was Sie sagen (Beispiele) | Automatischer Tool-Aufruf |
|--------------------------|--------------------------|
| "Microservices vs. Monolith", "Wie sollten wir das aufteilen?" | `search_knowledge("monolith microservice decomposition bounded context")` |
| "Ist diese Architektur in Ordnung?", "Bewerten Sie dieses Design" | `search_knowledge("layered architecture coupling responsibility")` |

### Praxisbeispiele

**Beispiel 1 — God Object erkennen:**

```
Sie: "UserService gerät außer Kontrolle. Es verwaltet Authentifizierung,
     Session-Management, E-Mail-Verifizierung und Passwort-Resets."

Claude (ruft automatisch search_knowledge auf):
  → Erkennt: God Object / Single Responsibility-Verletzung
  → Gibt zurück: SMELL-03 (God Object), LAW-001 (SRP), RF-018 (Extract Class)

Claude: "UserService zeigt God Object-Eigenschaften (SMELL-03).
Es verletzt das Single Responsibility Principle (LAW-001).

Empfohlene Refactorings:
1. Extract Class (RF-018) — SessionManager für Session-Logik erstellen
2. Move Method (RF-023) — E-Mail-Verifizierung zu EmailService verschieben
3. Extract Interface (RF-042) — AuthService-Interface definieren

Beginnen Sie mit #2 für schnelle Erfolge, danach #1 für strukturelle Verbesserung."
```

**Beispiel 2 — Performance-Untersuchung:**

```
Sie: "Die Produktlisten-API ist wirklich langsam. Einzelne Produkte laden
     problemlos, aber die Listenseite braucht 5+ Sekunden."

Claude (ruft automatisch search_knowledge auf):
  → Erkennt: N+1-Query-Problem
  → Gibt zurück: Performance-Patterns, Lazy Loading, Caching-Strategien

Claude: "Das klingt nach einem N+1-Query-Problem — jedes Produkt löst
einzelne DB-Aufrufe statt einer Batch-Abfrage aus.

Lösungen aus dem Wissensgraph:
1. Batch Loading / Eager Loading — Zugehörige Daten in einer Abfrage laden
2. Data Mapper (DP-019) — Abfragelogik von Domänenobjekten entkoppeln
3. Lazy Loading (RF-050) anwenden — Beziehungen nur bei Bedarf laden

Möchten Sie, dass ich den Code auf N+1-Patterns prüfe?"
```

**Beispiel 3 — Code-Review mit Snippet:**

```
Sie: [fügt Python-Code ein]
    "Können Sie das überprüfen? Irgendwas fühlt sich falsch an."

Claude (ruft automatisch analyze_code → suggest_refactorings auf):
  → analyze_code erkennt: Long Method (Konfidenz 0.87), Long Parameter List (0.92)
  → suggest_refactorings gibt priorisierte Vorschläge zurück

Claude: "2 Code-Smells gefunden:

  1. Long Method (SMELL-01) — 45 Zeilen, zyklomatische Komplexität 12
     → Extract Method (RF-001), Priorität 0.89

  2. Long Parameter List (SMELL-08) — 9 Parameter
     → Introduce Parameter Object (RF-029), Priorität 0.92

  Beginnen Sie mit RF-029 (Parameter Object) — es hat die höchste Priorität
  und macht das nachfolgende Extract Method einfacher."
```

---

## Nächste Schritte

1. **Agenten ausprobieren**: Fragen Sie den episteme-advisor "Sollte ich Singleton verwenden?"
2. **Code analysieren**: Funktion einfügen und code-reviewer um Smell-Prüfung bitten
3. **Graph erkunden**: episteme-researcher verwenden, um Pattern-Beziehungen zu finden
4. **Benutzerdefinierte Workflows**: Tools kombinieren (analyze → suggest → search)

Weitere Beispiele finden Sie unter:
- [Alcove-Integration](./alcove-integration.md) — Team-Wissen + Episteme
- [API-Referenz](./api.md) — REST-Endpunkte
