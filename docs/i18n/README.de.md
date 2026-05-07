<h1 align="center">Syntagma</h1>

<p align="center"><b>Wissensgraph fuer Software Engineering</b></p>

<p align="center"><sub>Syntagma (συνταγμα) — Griechisch fuer "organisiertes System" oder "Unterscheidungsvermoegen"</sub></p>

<p align="center">Ein offline-first, einzelbinary-Wissensgraph, der Entwurfsmuster, Refactoring-Techniken und Software-Prinzipien durch semantische Beziehungen verbindet.<br><b>Erstentwickelt fuer KI-Agenten</b> — integrieren Sie Software-Engineering-Expertise direkt in Claude Code, Cursor und andere MCP-kompatible Werkzeuge.</p>

<p align="center">Geschrieben in Rust · Einzelnes Binary · Vollstaendig offline</p>

---

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
  Deutsch |
  <a href="README.fr.md">Français</a> |
  <a href="README.zh-CN.md">简体中文</a> |
  <a href="README.zh-TW.md">繁體中文</a> |
  <a href="README.pt.md">Português</a> |
  <a href="README.es.md">Español</a> |
  <a href="README.hi.md">हिन्दी</a>
</p>



---

<img src="../assets/features.png" align="center" width="100%" alt="Syntagma Funktionsuebersicht" />

---

## Schnellstart

> **Voraussetzungen:** Rust 1.95+ ueber [rustup](https://rustup.rs) · **Kein Rust?** Siehe [Docker](#option-3-docker-kein-rust-erforderlich) oder [vorgefertigte Binaries](#option-4-vorgefertigte-binaries-kein-rust-erforderlich).

**1. Rust installieren (falls noch nicht installiert)**

| Betriebssystem | Befehl |
|----|---------|
| **macOS / Linux** | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **Windows** | [`rustup-init.exe`](https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe) herunterladen und ausfuehren |

Oeffnen Sie nach der Installation ein **neues Terminal** (oder fuehren Sie `source "$HOME/.cargo/env"` auf macOS/Linux aus).

**2. Syntagma installieren (erster Build dauert 3-5 Min.)**

```bash
cargo install --git https://github.com/epicsagas/Syntagma
```

**3. Daten laden + KI-Werkzeug verknuepfen**

```bash
syntagma install claude    # oder: cursor, codex, gemini
```

**4. Ueberpruefen**

```bash
syntagma --version
syntagma stats
```

Das war's. Starten Sie Claude Code neu und die Syntagma-Werkzeuge sind einsatzbereit.

### In 30 Sekunden ausprobieren

**Option A — CLI:** Auf jede Datei in Ihrem Projekt anwenden.

```bash
syntagma analyze src/domain/engine.rs
```

```
✓ 2 smells detected in src/domain/engine.rs

  SMELL-07 (Large Class) — RefactoringRanker, 743 lines
  → RF-018 Extract Class          priority 0.89  effort: medium
  → RF-001 Extract Method         priority 0.76  effort: small
  → Violates: LAW-001 Single Responsibility Principle

  SMELL-01 (Long Method) — rank_refactorings(), 58 lines
  → RF-001 Extract Method         priority 0.92  effort: small
  → Violates: LAW-001 SRP, LAW-004 DRY
```

**Option B — Claude Code:** Oeffnen Sie eine beliebige Datei in Ihrem Projekt und fragen Sie natuerlich.

```
Find code smells in this project and suggest refactorings.
```

Syntagma wird automatisch aktiviert — keine besondere Syntax erforderlich. Es beschreibt Ihr Problem im Wissensgraph und liefert bewertete, zitierfaehige Ergebnisse.

---

## Warum Syntagma?

LLMs wissen bereits, was das Strategie-Muster ist. Sie koennen SOLID-Prinzipien rezitieren, GoF-Muster auflisten und Code Smells erklaeren. Warum existiert also dieses Projekt?

**Die Luecke liegt nicht im Wissen — sondern im strukturierten, vernetzten Denken.**

Wenn Sie ein LLM fragen "wie repariere ich ein God Object?", erhalten Sie eine angemessene Antwort. Aber die Antwort aendert sich zwischen Gespraechen, es fehlt an Rueckverfolgbarkeit, und sie verbindet das Problem nicht mit seinen Ursachen oder downstream-Auswirkungen. Syntagma verwandelt isolierte Fakten in einen begehbaren Graphen, in dem jede Empfehlung begruendet, zitierfaehig und mit der breiteren Entwurfslandschaft verbunden ist.

### Worin unterscheidet sich dies von einem gut formulierten LLM-Prompt?

| | Gut formulierter LLM-Prompt | Syntagma + LLM |
|---|---|---|
| Proaktive Erkennung | Nur wenn der Benutzer die richtige Frage stellt | Wird automatisch bei Problembeschreibungen aktiviert |
| Token-Effizienz | Lange Erklaerungen + mehrere Nachfrage-Runden | Ein Werkzeugaufruf liefert ein strukturiertes Ergebnis |
| Beziehungs-Traversierung | Hoechstens ein Hop, oft halluziniert | Multi-Hop-Graph-Traversierung, verifiziert |
| Querverweise | Manuell, fehleranfaellig | Automatisiert ueber 201 semantische Beziehungen |
| Konsistenz | Variiert zwischen Gespraechem | Jedes Mal die gleiche strukturierte Antwort |
| Zitierfaehigkeit | "Ich denke, Sie sollten Extract Class verwenden" | "Extract Class (RF-018), Prioritaet 0.89" |
| Offline / Air-gapped | Benoetigt Internet fuer beste Ergebnisse | Vollstaendig lokal, einzelnes Binary |

### Wann ist das nuetzlich?

<details>
<summary><b>1. Wenn Ihr KI-Agent Probleme proaktiv erkennen soll, anstatt auf Fragen zu warten</b></summary>

Die MCP-Integration wird automatisch bei Problembeschreibungen aktiviert. Wenn ein Benutzer sagt "diese Klasse macht zu viel", muss der Agent nicht wissen, dass er nach God Object fragen soll — Syntagma ordnet die Beschwerde `SMELL-03` zu, zeigt bewertete Refactorings an und verfolgt die Verletzung bis zu den Grundprinzipien zurueck. Dies verwandelt eine vage Beschwerde in einen strukturierten Behebungsplan.
</details>

<details>
<summary><b>2. Wenn Sie den Token-Verbrauch reduzieren moechten — statt ihn fuer Erklaerungen zu verschwenden</b></summary>

Ohne Syntagma beantwortet ein LLM "wie repariere ich ein God Object?" indem es den Smell erklaert, Refactorings auflistet, SOLID-Prinzipien beschreibt und jede Option durchgeht — hunderte von Token pro Antwort. Mit Syntagma liefert ein einziger MCP-Werkzeugaufruf `SMELL-03 → RF-018 (0.89) → LAW-001`. Die gleiche Expertise zu einem Bruchteil des Token-Budgets.
</details>

<details>
<summary><b>3. Wenn Sie Code-Analyse brauchen, die mit der Behebung verbunden ist — nicht nur Erkennung</b></summary>

Werkzeuge wie SonarQube erkennen Smells. LLMs koennen Muster vorschlagen. Syntagma macht beides und verbindet sie: Long Method erkennen → zu den verletzten Prinzipien zurueckverfolgen → die Refactorings bewerten, die es loesen → zeigen, welche Muster diese Refactorings durchsetzen.
</details>

<details>
<summary><b>4. Wenn isoliertes Musterwissen nicht ausreicht — Sie brauchen die Beziehungen</b></summary>

Zu wissen, was Extract Method macht, ist Grundvoraussetzung. Zu wissen, dass es Long Method (SMELL-01) *loest*, welches Single Responsibility (LAW-001) *verletzt*, welches vom Facade-Pattern (DP-012) *durchgesetzt* wird — das ist eine Denk-Kette, die ein LLM nicht zuverlaessig selbst konstruieren kann. Syntagmas 201 semantische Beziehungen ermoeglichen es KI-Agenten, diese Pfade deterministisch zu traversieren.
</details>

<details>
<summary><b>5. Wenn Sie Architekturentscheidungen treffen und Beweise brauchen, keine Meinungen</b></summary>

"Soll ich Microservices verwenden?" — Syntagma verbindet die Frage mit Conways Gesetz (LAW-017), SRP (LAW-001) und dem Strangler-Fig-Muster (DP-026) und zeigt dann, wie sie zusammenhaengen. Entscheidungen werden auf Engineering-Prinzipien zurueckfuehrbar, nicht auf Blogbeitraege.
</details>

<details>
<summary><b>6. Wenn Sie konsistente, zitierfaehige Engineering-Beratung brauchen — keine halluzinierten Empfehlungen</b></summary>

Jedes Ergebnis verweist auf explizite Entitaets-IDs (`DP-005`, `RF-001`, `LAW-021`). Empfehlungen kommen mit Prioritaetsbewertungen und Schaetzungen fuer den Aufwand. Die gleiche Abfrage liefert immer die gleiche strukturierte Antwort.
</details>

<details>
<summary><b>7. Wenn Sie in einer air-gapped oder beschraenkten Netzwerkumgebung arbeiten</b></summary>

Syntagma laeuft vollstaendig offline: einzelnes Binary, lokale SQLite-Datenbank, lokale Embeddings ueber fastembed (ONNX Runtime). Keine Telemetrie, kein Phone-Home, keine externen API-Aufrufe. Ihr Code und Ihre Analyseergebnisse verlassen niemals Ihren Rechner.
</details>

---

## Installation

### Option 1: Ein Befehl (Empfohlen)

```bash
# Erster Build dauert 3-5 Minuten — das ist normal
cargo install --git https://github.com/epicsagas/Syntagma
syntagma install claude    # laedt Daten + verknuepft MCP + installiert Agenten
```

> Starten Sie nach `syntagma install claude` **Claude Code neu**, damit die MCP-Werkzeuge und Agenten erscheinen.

### Option 2: Aus dem Quellcode

```bash
git clone https://github.com/epicsagas/Syntagma.git
cd Syntagma && cargo build --release
```

Fuehren Sie dann das Binary fuer Ihre Plattform aus:

| Plattform | Befehl |
|----------|---------|
| **macOS / Linux** | `./target/release/syntagma install --local claude` |
| **Windows** | `.\target\release\syntagma.exe install --local claude` |

### Option 3: Docker (Kein Rust erforderlich)

```bash
docker-compose up -d
```

Fuegen Sie dies zu Ihrer MCP-Konfigurationsdatei hinzu:

| Werkzeug | Pfad zur Konfigurationsdatei |
|------|-----------------|
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

### Option 4: Vorgefertigte Binaries (Kein Rust erforderlich)

Laden Sie das neueste Binary fuer Ihre Plattform von [GitHub Releases](https://github.com/epicsagas/Syntagma/releases) herunter:

| Plattform | Datei |
|----------|------|
| **macOS** (Apple Silicon) | `syntagma-aarch64-apple-darwin.tar.gz` |
| **macOS** (Intel) | `syntagma-x86_64-apple-darwin.tar.gz` |
| **Linux** (x86_64) | `syntagma-x86_64-unknown-linux-gnu.tar.gz` |
| **Linux** (ARM64) | `syntagma-aarch64-unknown-linux-gnu.tar.gz` |
| **Windows** (x86_64) | `syntagma-x86_64-pc-windows-msvc.zip` |

```bash
# macOS / Linux
tar xzf syntagma-*.tar.gz
sudo mv syntagma /usr/local/bin/

# Windows — ZIP entpacken und syntagma.exe zum PATH hinzufuegen
```

Dann installieren:
```bash
syntagma install claude    # oder: cursor, codex, gemini
```

### Ueberpruefen

```bash
syntagma --version
syntagma stats
syntagma explore "strategy pattern"    # den Wissensgraph erkunden
```

---

## MCP-Werkzeuge & Agenten

> **Was ist MCP?** Das [Model Context Protocol](https://modelcontextprotocol.io) ist ein offener Standard, der es KI-Werkzeugen ermoeglicht, externe Dienste aufzurufen. Syntagma stellt seinen Wissensgraph als MCP-Werkzeuge bereit, die Claude Code, Cursor und andere kompatible Editoren automatisch aufrufen koennen.

### 6 MCP-Werkzeuge

| Werkzeug | Zweck | Anwendungsbeispiel |
|------|---------|-------------|
| **`search_knowledge`** | Semantische Suche ueber alle Entitaeten | "Muster fuer Retry-Logik finden" |
| **`get_entity`** | Details fuer eine bestimmte Entitaet nach ID | "Strategy Pattern (DP-023) erklaeren" |
| **`get_neighbors`** | Verwandte Entitaeten erkunden | "Welche Refactorings loesen Long Method?" |
| **`find_path`** | Verbindung zwischen zwei Entitaeten finden | "Wie haengt SRP mit Extract Class zusammen?" |
| **`analyze_code`** | Code Smells ueber Regex/AST-Analyse erkennen | "Diesen Zahlungsvalidierungscode pruefen" |
| **`suggest_refactorings`** | Bewertete Refactoring-Vorschlaege | "Was sollte ich in dieser Klasse refactoren?" |

### 4 Spezialisierte Agenten (Vernetztes System)

Agenten arbeiten zusammen — jede Analyse endet mit **Naechste Schritte**-Optionen, die an andere Agenten uebergeben werden.

| Agent | Wann zu verwenden | Hauptfaehigkeit | Uebergibt an |
|-------|-------------|----------------|--------------|
| **`code-reviewer`** | Code Smells, SOLID-Verletzungen | Kausalitaetsanalyse (Ursache → downstream-Symptome) | advisor, architecture-analyst, refactoring-expert |
| **`syntagma-advisor`** | Engineering-Entscheidungen, Trade-offs | Multi-Entitaets-Trade-off-Ketten mit Aktionsplaenen | code-reviewer, architecture-analyst, researcher |
| **`syntagma-researcher`** | Wissensgraph-Erkundung | Verbindungskarten zwischen Mustern, Prinzipien, Smells | advisor, code-reviewer |
| **`architecture-analyst`** | Architekturbewertung anhand von Prinzipien | Compliance-Bewertung mit risikogewichteter Einschaetzung | advisor, code-reviewer, researcher |

**Workflow-Beispiel**: `code-reviewer` erkennt God Object → verfolgt Kausalitaet zu 3 downstream-Smells → bietet "RF-018 anwenden" (→ refactoring-expert) oder "Ursache vertiefen" (→ syntagma-advisor) oder "Architekturpruefung" (→ architecture-analyst).

[Vollstaendiger MCP-Integrationsleitfaden](../../docs/mcp-integration-guide.md)

---

## CLI-Verwendung

```bash
# Code auf Smells analysieren
syntagma analyze my_code.py --language python --json
syntagma infer my_code.py

# Den Wissensgraph erkunden
syntagma explore "strategy pattern"
syntagma graph path DP-005 RF-001   # z.B. Factory Method → Extract Method

# Den RAG-Index erstellen
syntagma build

# Server starten
syntagma api              # REST API auf :8000
syntagma mcp --http       # MCP-Server auf :43175
syntagma web --port 8080  # Web-UI (interaktiver Graph-Explorer)

# Distributions-Paketierung
syntagma dist --out-dir release/
```

---

## Funktionen

### Wissensbasis
- **22 GoF-Entwurfsmuster** — Vollstaendiger Katalog mit Praxisbeispielen
- **66 Refactoring-Techniken** — Aus Fowlers Katalog mit Codebeispielen
- **56 Software-Prinzipien & Gesetze** — SOLID, Conways Gesetz, CAP-Theorem usw.
- **17 Code Smell-Typen** — Long Method, God Object, Feature Envy usw. ¹
- **201 Semantische Beziehungen** — "loest", "durchsetzt", "verletzt", "bezieht_sich_auf"

### KI-zentriertes Design
- **MCP-Integration** — 6 spezialisierte Werkzeuge fuer hochwertige KI-Agenten-Interaktion
- **4 Vernetzte Agenten** — Kausalitaetsanalyse, interaktive Nachfragen und agentenuebergreifende Uebergaben
- **10 Sprachen unterstuetzt** — Python (AST), Java, TypeScript, Go, Rust, C++, C#, PHP, Ruby, Kotlin
- **Deterministische Analyse** — AST-basierte Python-Erkennung + regex-basierte Mehrsprachenunterstuetzung
- **Zitierfaehiges Wissen** — Jedes Ergebnis verweist auf explizite Entitaets-IDs (z.B. `RF-001`, `LAW-021`)
- **Workflow-Ketten** — Mehrstufige Pipelines: Code-Review → Kausalitaetsanalyse → Refactoring → Verifizierung

### Produktionsbereit
- **REST API** — 17 Endpunkte mit Authentifizierung und Ratenbegrenzung
- **Einzelnes Binary** — Keine Laufzeitabhaengigkeiten, plattformuebergreifend
- **Lokale Embeddings** — fastembed (ONNX Runtime) fuer zero-config semantische Suche
- **Interaktive Visualisierung** — Webbasierte Graph-Explorer (`syntagma web`)
- **Docker-Unterstuetzung** — Mehrstufiger Build mit Health-Checks
- **Monitoring** — Prometheus-Metrik-Endpunkt

> ¹ Duplicate Code (SMELL-13) und Shotgun Surgery (SMELL-09) erfordern Multi-Datei-Kontext und werden im Einzeldateimodus uebersprungen.

---

## Dokumentation

| Dokument | Beschreibung |
|----------|-------------|
| [Schnellstart](../../QUICKSTART.md) | Schritt-fuer-Schritt-Einrichtung, erster Start, Fehlerbehebung |
| [MCP-Integrationsleitfaden](../../docs/mcp-integration-guide.md) | Werkzeugreferenz, Agenten-Beispiele, Konversationsablaeufe |
| [API-Referenz](../../docs/api.md) | REST-Endpunkte, Authentifizierung, Beispiele |
| [Distribution](../../docs/distribution.md) | Release-Paketierung und Bereitstellung |
| [Entwicklung & Mitwirken](../../DEVELOPMENT.md) | Architektur, wie man beitraegt |
| [Aenderungsprotokoll](../../CHANGELOG.md) | Release-Historie und Versionshinweise |

---

## Konfiguration

### Umgebungsvariablen

```bash
# Datenverzeichnisse
SYNTAGMA_DATA_DIR=~/.syntagma/data
SYNTAGMA_DB_PATH=~/.syntagma/db/syntagma.db

# API-Server
SYNTAGMA_API_HOST=0.0.0.0
SYNTAGMA_API_PORT=8000
SYNTAGMA_API_KEY=your-secret-key

# MCP-Server
SYNTAGMA_MCP_HOST=127.0.0.1
SYNTAGMA_MCP_PORT=43175
```

---

## Fehlerbehebung

**`syntagma`-Befehl nach der Installation nicht gefunden**

| Plattform | Loesung |
|----------|-----|
| **macOS / Linux** | `export PATH="$HOME/.cargo/bin:$PATH"` — zu `~/.bashrc` oder `~/.zshrc` hinzufuegen fuer dauerhafte Wirkung |
| **Windows** | `%USERPROFILE%\.cargo\bin` zum System-PATH hinzufuegen oder ein neues Terminal oeffnen |

**MCP-Werkzeuge erscheinen nicht in Claude Code / Cursor**

Starten Sie den Editor nach dem Ausfuehren von `syntagma install` neu. Wenn sie immer noch fehlen, pruefen Sie, ob die Konfiguration geschrieben wurde:
```bash
cat ~/.claude.json   # Claude Code
```

**Port bereits in Verwendung**
```bash
syntagma mcp --http --port 43176   # einen anderen Port verwenden
```

**Langsamer erster Start**

Syntagma erstellt beim ersten Start einen lokalen Embedding-Index. Dies dauert 30-60 Sekunden und ist ein einmaliger Aufwand. Nachfolgende Starts erfolgen sofort.

**Kompilierungsfehler bei `cargo install`**

Stellen Sie sicher, dass Rust 1.95+ installiert ist:
```bash
rustup update stable
rustup show   # aktive Toolchain bestaetigen
```

> Weitere Hilfe: [QUICKSTART.md Fehlerbehebungsabschnitt](../../QUICKSTART.md#troubleshooting) · [Issue eroeffnen](https://github.com/epicsagas/Syntagma/issues)

---

## Roadmap

- [ ] **Benutzerdefinierte Entitaeten** — Team-spezifische Muster/Smells hinzufuegen
- [ ] **Interaktive Tutorials** — Anwendungsinterne gefuehrte Touren fuer MCP-Werkzeuge
- [ ] **Mehrsprachige Metadaten** — Entitaetstitel und Zusammenfassungen auf Koreanisch, Japanisch, Chinesisch (README-Uebersetzungen bereits abgeschlossen)
- [ ] **MCP-Werkzeugbeschreibungen** — Verbesserte Beschreibungen als Ersatz fuer IDE-spezifische Plugins
- [ ] **Team-Metriken** — Aggregierte Musternutzung ueber die Organisation hinweg

---

## Mitwirken

Beitraege sind willkommen! Siehe [DEVELOPMENT.md](../../DEVELOPMENT.md) fuer die Architekturuebersicht und den Leitfaden zum Mitwirken.

```bash
# Tests ausfuehren
cargo test

# Linten
cargo clippy -- -D warnings

# Formatieren
cargo fmt
```

Fragen? [Diskussion eroeffnen](https://github.com/epicsagas/Syntagma/discussions) oder [Issue melden](https://github.com/epicsagas/Syntagma/issues).

---

## Lizenz

Apache 2.0 — siehe [LICENSE](../../LICENSE) fuer Details.
