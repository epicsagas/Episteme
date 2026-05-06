<h1 align="center">Syntagma</h1>

<p align="center"><b>Graphe de connaissances pour le genie logiciel</b></p>

<p align="center"><sub>Syntagma (συνταγμα) — du grec « systeme organise » ou « discernement »</sub></p>

<p align="center">Un graphe de connaissances hors-ligne, en un seul binaire, qui connecte les motifs de conception, les techniques de refactoring et les lois du logiciel par des relations semantiques.<br><b>Concu d'abord pour les agents IA</b> — integrez l'expertise en genie logiciel directement dans Claude Code, Cursor et autres outils compatibles MCP.</p>

<p align="center">Ecrit en Rust · Binaire unique · Entierement hors-ligne</p>

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
  <a href="README.de.md">Deutsch</a> |
  Français |
  <a href="README.zh-CN.md">简体中文</a> |
  <a href="README.zh-TW.md">繁體中文</a> |
  <a href="README.pt.md">Português</a> |
  <a href="README.es.md">Español</a> |
  <a href="README.hi.md">हिन्दी</a>
</p>



---

<img src="../assets/features.png" align="center" width="100%" alt="Apercu des fonctionnalites de Syntagma" />

---

## Demarrage rapide

> **Prerequis :** Rust 1.95+ via [rustup](https://rustup.rs) · **Pas de Rust ?** Voir [Docker](#option-3-docker-rust-non-requis) ou [binaires precompiles](#option-4-binaires-precompiles-rust-non-requis).

**1. Installer Rust (si ce n'est pas deja fait)**

| OS | Commande |
|----|----------|
| **macOS / Linux** | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **Windows** | Telecharger et executer [`rustup-init.exe`](https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe) |

Apres l'installation, ouvrez un **nouveau terminal** (ou executez `source "$HOME/.cargo/env"` sur macOS/Linux).

**2. Installer Syntagma (la premiere compilation prend 3 a 5 min)**

```bash
cargo install --git https://github.com/epicsagas/Syntagma
```

**3. Initialiser les donnees + connecter votre outil IA**

```bash
syntagma install claude    # ou : cursor, codex, gemini
```

**4. Verifier**

```bash
syntagma --version
syntagma stats
```

C'est tout. Redemarrez Claude Code et les outils Syntagma sont prets.

### Essayez en 30 secondes

**Option A — CLI :** Pointez vers n'importe quel fichier de votre projet.

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

**Option B — Claude Code :** Ouvrez n'importe quel fichier de votre projet et demandez naturellement.

```
Find code smells in this project and suggest refactorings.
```

Syntagma se declenche automatiquement — aucune syntaxe speciale n'est necessaire. Il mappe votre description au graphe de connaissances et renvoie des resultats classes et citables.

---

## Pourquoi Syntagma ?

Les LLM connaissent deja le motif Strategy. Ils peuvent reciter les principes SOLID, lister les motifs GoF et expliquer les code smells. Alors pourquoi ce projet existe-t-il ?

**Le manque n'est pas la connaissance — c'est le raisonnement structure et connecte.**

Quand vous demandez a un LLM « comment corriger un God Object ? », il vous donne une reponse raisonnable. Mais cette reponse change entre les conversations, manque de tracabilite et ne connecte pas le probleme a ses causes racines ou a ses consequences en aval. Syntagma transforme les faits isoles en un graphe traversable ou chaque recommandation est fondee, citable et connectee au panorama de conception global.

### En quoi cela differe-t-il d'un bon prompt LLM ?

| | Prompt LLM bien elabore | Syntagma + LLM |
|---|---|---|
| Detection proactive | Uniquement si l'utilisateur pose la bonne question | Se declenche automatiquement sur les descriptions de problemes |
| Efficacite en tokens | Longues explications + plusieurs tours de suivi | Un appel d'outil renvoie un resultat structure |
| Traversee de relations | Un saut au mieux, souvent hallucine | Traversee de graphe multi-sauts, verifiee |
| References croisees | Manuelles, sujettes aux erreurs | Automatisees via 201 relations semantiques |
| Coherence | Varie entre les conversations | Toujours la meme reponse structuree |
| Citabilite | « Je pense que vous devriez utiliser Extract Class » | « Extract Class (RF-018), priorite 0.89 » |
| Hors-ligne / Reseau coupe | Necessite Internet pour de meilleurs resultats | Entierement local, binaire unique |

### Quand est-ce utile ?

<details>
<summary><b>1. Quand votre agent IA devrait detecter proactivement les problemes, et non attendre qu'on le lui demande</b></summary>

L'integration MCP se declenche automatiquement sur les descriptions de problemes. Quand un utilisateur dit « cette classe fait trop de choses », l'agent n'a pas besoin de savoir poser la question sur le God Object — Syntagma mappe la plainte a `SMELL-03`, affiche les refactorings classes et retrace la violation jusqu'aux principes fondamentaux. Cela transforme une plainte vague en un plan de remediation structure.
</details>

<details>
<summary><b>2. Quand vous voulez reduire la consommation de tokens — pas la gaspiller en explications</b></summary>

Sans Syntagma, un LLM repond a « comment corriger un God Object ? » en expliquant le smell, en listant les refactorings, en decrivant les principes SOLID et en parcourant chaque option — des centaines de tokens par reponse. Avec Syntagma, un seul appel d'outil MCP renvoie `SMELL-03 → RF-018 (0.89) → LAW-001`. La meme expertise pour une fraction du budget de tokens.
</details>

<details>
<summary><b>3. Quand vous avez besoin d'une analyse de code connectee a la remediation — pas seulement a la detection</b></summary>

Des outils comme SonarQube detectent les smells. Les LLM peuvent suggerer des motifs. Syntagma fait les deux et les connecte : detecter Long Method → tracer les lois violees → classer les refactorings qui les resolvent → montrer quels motifs renforcent ces refactorings.
</details>

<details>
<summary><b>4. Quand la connaissance isolee des motifs ne suffit pas — vous avez besoin des relations</b></summary>

Savoir ce que fait Extract Method est un prerequis de base. Savoir qu'il *resout* Long Method (SMELL-01), qui *viole* Single Responsibility (LAW-001), qui est *renforce par* le motif Facade (DP-012) — c'est une chaine de raisonnement qu'un LLM ne peut pas construire de maniere fiable par lui-meme. Les 201 relations semantiques de Syntagma permettent aux agents IA de traverser ces chemins de maniere deterministe.
</details>

<details>
<summary><b>5. Quand vous prenez des decisions d'architecture et avez besoin de preuves, pas d'opinions</b></summary>

« Dois-je utiliser les microservices ? » — Syntagma connecte la question a la Loi de Conway (LAW-017), au SRP (LAW-001) et au motif Strangler Fig (DP-026), puis montre comment ils sont relies. Les decisions deviennent tracables vers des lois d'ingenierie, pas des articles de blog.
</details>

<details>
<summary><b>6. Quand vous avez besoin de conseils d'ingenierie coherents et citables — pas de recommandations hallucinees</b></summary>

Chaque resultat reference des identifiants d'entite explicites (`DP-005`, `RF-001`, `LAW-021`). Les recommandations sont accompagnees de scores de priorite et d'estimations d'effort. La meme requete renvoie toujours la meme reponse structuree.
</details>

<details>
<summary><b>7. Quand vous travaillez dans un environnement deconnecte ou sur un reseau restreint</b></summary>

Syntagma fonctionne entierement hors-ligne : binaire unique, base de donnees SQLite locale, embeddings locaux via fastembed (ONNX Runtime). Pas de telemetrie, pas d'appel a un serveur externe, pas d'API externe. Votre code et vos resultats d'analyse ne quittent jamais votre machine.
</details>

---

## Installation

### Option 1 : Une seule commande (Recommandee)

```bash
# La premiere compilation prend 3 a 5 minutes — c'est normal
cargo install --git https://github.com/epicsagas/Syntagma
syntagma install claude    # initialise les donnees + connecte MCP + installe les agents
```

> Apres `syntagma install claude`, **redemarrez Claude Code** pour que les outils MCP et les agents apparaissent.

### Option 2 : A partir des sources

```bash
git clone https://github.com/epicsagas/Syntagma.git
cd Syntagma && cargo build --release
```

Puis executez le binaire pour votre plateforme :

| Plateforme | Commande |
|------------|----------|
| **macOS / Linux** | `./target/release/syntagma install --local claude` |
| **Windows** | `.\target\release\syntagma.exe install --local claude` |

### Option 3 : Docker (Rust non requis)

```bash
docker-compose up -d
```

Ajoutez a votre fichier de configuration MCP :

| Outil | Chemin du fichier de configuration |
|-------|-----------------------------------|
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

### Option 4 : Binaires precompiles (Rust non requis)

Telechargez le dernier binaire pour votre plateforme depuis [GitHub Releases](https://github.com/epicsagas/Syntagma/releases) :

| Plateforme | Fichier |
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

# Windows — extrayez le zip et ajoutez syntagma.exe a votre PATH
```

Puis installez :
```bash
syntagma install claude    # ou : cursor, codex, gemini
```

### Verifier

```bash
syntagma --version
syntagma stats
syntagma explore "strategy pattern"    # explorer le graphe de connaissances
```

---

## Outils MCP et Agents

> **Qu'est-ce que MCP ?** Le [Model Context Protocol](https://modelcontextprotocol.io) est une norme ouverte qui permet aux outils IA d'appeler des services externes. Syntagma expose son graphe de connaissances sous forme d'outils MCP que Claude Code, Cursor et autres editeurs compatibles peuvent appeler automatiquement.

### 6 outils MCP

| Outil | Objectif | Exemple d'utilisation |
|-------|----------|----------------------|
| **`search_knowledge`** | Recherche semantique sur toutes les entites | « Trouver des motifs pour la logique de retry » |
| **`get_entity`** | Obtenir les details d'une entite specifique par ID | « Expliquer le motif Strategy (DP-023) » |
| **`get_neighbors`** | Explorer les entites liees | « Quels refactorings resolvent Long Method ? » |
| **`find_path`** | Trouver une connexion entre deux entites | « Comment le SRP est-il lie a Extract Class ? » |
| **`analyze_code`** | Detecter les code smells via analyse regex/AST | « Examiner ce code de validation de paiement » |
| **`suggest_refactorings`** | Suggestions de refactoring classees | « Que devrais-je refactorer dans cette classe ? » |

### 4 Agents specialises (Reseau connecte)

Les agents travaillent ensemble — chaque analyse se termine par des options **Prochaines etapes** qui passent le relais a d'autres agents.

| Agent | Quand l'utiliser | Capacite cle | Passe le relais a |
|-------|-----------------|-------------|-------------------|
| **`code-reviewer`** | Code smells, violations SOLID | Analyse de causalite (cause racine → symptomes en aval) | advisor, architecture-analyst, refactoring-expert |
| **`syntagma-advisor`** | Decisions d'ingenierie, compromis | Chaines de compromis multi-entites avec plans d'action | code-reviewer, architecture-analyst, researcher |
| **`syntagma-researcher`** | Exploration du graphe de connaissances | Cartes de connexions entre motifs, lois, smells | advisor, code-reviewer |
| **`architecture-analyst`** | Evaluation d'architecture par rapport aux lois | Score de conformite avec evaluation ponderee par les risques | advisor, code-reviewer, researcher |

**Exemple de flux de travail** : `code-reviewer` detecte un God Object → trace la causalite vers 3 smells en aval → offre « Appliquer RF-018 » (→ refactoring-expert) ou « Analyse approfondie de la cause racine » (→ syntagma-advisor) ou « Verification d'architecture » (→ architecture-analyst).

[Guide complet d'integration MCP](../mcp-integration-guide.md)

---

## Utilisation CLI

```bash
# Analyser le code pour les smells
syntagma analyze my_code.py --language python --json
syntagma infer my_code.py

# Explorer le graphe de connaissances
syntagma explore "strategy pattern"
syntagma graph path DP-005 RF-001   # ex. Factory Method → Extract Method

# Construire l'index RAG
syntagma build

# Demarrer les serveurs
syntagma api              # REST API sur :8000
syntagma mcp --http       # Serveur MCP sur :43175
syntagma web --port 8080  # Interface Web (explorateur de graphe interactif)

# Packaging de distribution
syntagma dist --out-dir release/
```

---

## Fonctionnalites

### Base de connaissances
- **22 motifs de conception GoF** — Catalogue complet avec exemples concrets
- **66 techniques de refactoring** — Du catalogue de Fowler avec exemples de code
- **56 lois et principes du logiciel** — SOLID, Loi de Conway, Theoreme CAP, etc.
- **17 types de code smells** — Long Method, God Object, Feature Envy, etc. ¹
- **201 relations semantiques** — « resout », « renforce », « viole », « se rapporte a »

### Conception AI-First
- **Integration MCP** — 6 outils specialises pour une interaction haute fidelite avec les agents IA
- **4 agents connectes** — Analyse de causalite, suivi interactif et transferts entre agents
- **Prise en charge de 10 langages** — Python (AST), Java, TypeScript, Go, Rust, C++, C#, PHP, Ruby, Kotlin
- **Analyse deterministe** — Detection Python basee sur l'AST + support multi-langage base sur les regex
- **Connaissances citables** — Chaque resultat est lie a des identifiants d'entite explicites (ex. `RF-001`, `LAW-021`)
- **Chaines de flux de travail** — Pipelines multi-etapes : Revue de code → Analyse de causalite → Refactoring → Verification

### Pret pour la production
- **REST API** — 17 points d'acces avec authentification et limitation de debit
- **Binaire unique** — Pas de dependances d'execution, multiplateforme
- **Embeddings locaux** — fastembed (ONNX Runtime) pour la recherche semantique sans configuration
- **Visualisation interactive** — Explorateur de graphe Web (`syntagma web`)
- **Support Docker** — Build multi-etapes avec verifications de sante
- **Supervision** — Point d'acces pour metriques Prometheus

> ¹ Duplicate Code (SMELL-13) et Shotgun Surgery (SMELL-09) necessitent un contexte multi-fichiers et sont ignores en mode fichier unique.

---

## Documentation

| Document | Description |
|----------|-------------|
| [Demarrage rapide](../QUICKSTART.md) | Installation etape par etape, premiere execution, depannage |
| [Guide d'integration MCP](../mcp-integration-guide.md) | Reference des outils, exemples d'agents, flux de conversation |
| [Reference API](../api.md) | Points d'acces REST, authentification, exemples |
| [Distribution](../distribution.md) | Packaging de release et deploiement |
| [Developpement et Contribution](../DEVELOPMENT.md) | Architecture, comment contribuer |
| [Journal des modifications](../CHANGELOG.md) | Historique des versions et notes de mise a jour |

---

## Configuration

### Variables d'environnement

```bash
# Emplacements des donnees
SYNTAGMA_DATA_DIR=~/.syntagma/data
SYNTAGMA_DB_PATH=~/.syntagma/db/syntagma.db

# Serveur API
SYNTAGMA_API_HOST=0.0.0.0
SYNTAGMA_API_PORT=8000
SYNTAGMA_API_KEY=your-secret-key

# Serveur MCP
SYNTAGMA_MCP_HOST=127.0.0.1
SYNTAGMA_MCP_PORT=43175
```

---

## Depannage

**Commande `syntagma` introuvable apres l'installation**

| Plateforme | Solution |
|------------|----------|
| **macOS / Linux** | `export PATH="$HOME/.cargo/bin:$PATH"` — ajoutez a `~/.bashrc` ou `~/.zshrc` pour rendre persistant |
| **Windows** | Ajoutez `%USERPROFILE%\.cargo\bin` au PATH systeme, ou ouvrez un nouveau terminal |

**Les outils MCP n'apparaissent pas dans Claude Code / Cursor**

Redemarrez l'editeur apres avoir execute `syntagma install`. Si toujours absent, verifiez que la configuration a ete ecrite :
```bash
cat ~/.claude.json   # Claude Code
```

**Port deja utilise**
```bash
syntagma mcp --http --port 43176   # utiliser un port different
```

**Premier demarrage lent**

Syntagma construit un index d'embeddings local au premier lancement. Cela prend 30 a 60 secondes et est un cout ponctuel. Les demarrages suivants sont instantanes.

**Erreurs de compilation pendant `cargo install`**

Assurez-vous que Rust 1.95+ est installe :
```bash
rustup update stable
rustup show   # confirmer la toolchain active
```

> Plus d'aide : [Section depannage de QUICKSTART.md](../QUICKSTART.md#troubleshooting) · [Ouvrir un ticket](https://github.com/epicsagas/Syntagma/issues)

---

## Feuille de route

- [ ] **Tutoriels interactifs** — Visites guidees integrees pour les outils MCP
- [ ] **Metriques d'equipe** — Agréger l'utilisation des motifs au sein de l'organisation
- [ ] **Entites personnalisees** — Ajouter des motifs/smells specifiques a l'equipe
- [ ] **Plugins IDE** — Integrations natives VSCode, IntelliJ
- [ ] **Documentation multilingue** — Base de connaissances en coreen, japonais, chinois

---

## Contribuer

Les contributions sont les bienvenues ! Voir [DEVELOPMENT.md](../DEVELOPMENT.md) pour l'aperçu de l'architecture et le guide de contribution.

```bash
# Executer les tests
cargo test

# Lint
cargo clippy -- -D warnings

# Formatage
cargo fmt
```

Des questions ? [Ouvrir une discussion](https://github.com/epicsagas/Syntagma/discussions) ou [creer un ticket](https://github.com/epicsagas/Syntagma/issues).

---

## Licence

Apache 2.0 — voir [LICENSE](../LICENSE) pour plus de details.
