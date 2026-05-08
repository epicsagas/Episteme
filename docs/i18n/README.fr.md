<h1 align="center">Episteme</h1>

<p align="center"><b>Graphe de connaissances pour le genie logiciel</b></p>

<p align="center"><sub>Episteme (συνταγμα) — du grec « systeme organise » ou « discernement »</sub></p>

<p align="center">Un graphe de connaissances hors-ligne, en un seul binaire, qui connecte les motifs de conception, les techniques de refactoring et les lois du logiciel par des relations semantiques.<br><b>Concu d'abord pour les agents IA</b> — integrez l'expertise en genie logiciel directement dans Claude Code, Cursor et autres outils compatibles MCP.</p>

<p align="center">Ecrit en Rust · Binaire unique · Entierement hors-ligne</p>

---

<p align="center">
    <a href="https://github.com/epicsagas/Episteme/actions"><img src="https://img.shields.io/github/actions/workflow/status/epicsagas/Episteme/ci.yml?branch=main&label=CI" alt="CI" /></a>
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.95+-orange.svg" alt="rust-lang" /></a>
    <a href="https://crates.io/crates/episteme"><img src="https://img.shields.io/badge/crates.io-v0.1.0-orange.svg" alt="crates.io" /></a>
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

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/features.png">
  <img src="../assets/features.png" align="center" width="100%" alt="Apercu des fonctionnalites de Episteme" />
</picture>

---

## Demarrage rapide

> **Prerequis :** Rust 1.95+ via [rustup](https://rustup.rs) · **Pas de Rust ?** Voir [Docker](#option-3-docker-rust-non-requis) ou [binaires precompiles](#option-4-binaires-precompiles-rust-non-requis).

**1. Installer Rust (si ce n'est pas deja fait)**

| OS | Commande |
|----|----------|
| **macOS / Linux** | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **Windows** | Telecharger et executer [`rustup-init.exe`](https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe) |

Apres l'installation, ouvrez un **nouveau terminal** (ou executez `source "$HOME/.cargo/env"` sur macOS/Linux).

**2. Installer Episteme (la premiere compilation prend 3 a 5 min)**

```bash
# Install cargo-binstall if missing
cargo install cargo-binstall

# Fast path (downloads prebuilt binary when available)
cargo binstall episteme

# Fallback (build from source)
cargo install --git https://github.com/epicsagas/Episteme
```

**3. Initialiser les donnees + connecter votre outil IA**

```bash
epis install claude    # ou : cursor, codex, gemini
```

**4. Verifier**

```bash
epis --version
epis stats
```

C'est tout. Redemarrez Claude Code et les outils Episteme sont prets.

### Essayez en 30 secondes

**Option A — CLI :** Pointez vers n'importe quel fichier de votre projet.

```bash
epis analyze src/domain/engine.rs
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

Episteme se declenche automatiquement — aucune syntaxe speciale n'est necessaire. Il mappe votre description au graphe de connaissances et renvoie des resultats classes et citables.

---

## Pourquoi Episteme ?

Les LLM connaissent deja le motif Strategy. Ils peuvent reciter les principes SOLID, lister les motifs GoF et expliquer les code smells. Alors pourquoi ce projet existe-t-il ?

**Le manque n'est pas la connaissance — c'est le raisonnement structure et connecte.**

Quand vous demandez a un LLM « comment corriger un God Object ? », il vous donne une reponse raisonnable. Mais cette reponse change entre les conversations, manque de tracabilite et ne connecte pas le probleme a ses causes racines ou a ses consequences en aval. Episteme transforme les faits isoles en un graphe traversable ou chaque recommandation est fondee, citable et connectee au panorama de conception global.

### En quoi cela differe-t-il d'un bon prompt LLM ?

| | Prompt LLM bien elabore | Episteme + LLM |
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

L'integration MCP se declenche automatiquement sur les descriptions de problemes. Quand un utilisateur dit « cette classe fait trop de choses », l'agent n'a pas besoin de savoir poser la question sur le God Object — Episteme mappe la plainte a `SMELL-03`, affiche les refactorings classes et retrace la violation jusqu'aux principes fondamentaux. Cela transforme une plainte vague en un plan de remediation structure.
</details>

<details>
<summary><b>2. Quand vous voulez reduire la consommation de tokens — pas la gaspiller en explications</b></summary>

Sans Episteme, un LLM repond a « comment corriger un God Object ? » en expliquant le smell, en listant les refactorings, en decrivant les principes SOLID et en parcourant chaque option — des centaines de tokens par reponse. Avec Episteme, un seul appel d'outil MCP renvoie `SMELL-03 → RF-018 (0.89) → LAW-001`. La meme expertise pour une fraction du budget de tokens.
</details>

<details>
<summary><b>3. Quand vous avez besoin d'une analyse de code connectee a la remediation — pas seulement a la detection</b></summary>

Des outils comme SonarQube detectent les smells. Les LLM peuvent suggerer des motifs. Episteme fait les deux et les connecte : detecter Long Method → tracer les lois violees → classer les refactorings qui les resolvent → montrer quels motifs renforcent ces refactorings.
</details>

<details>
<summary><b>4. Quand la connaissance isolee des motifs ne suffit pas — vous avez besoin des relations</b></summary>

Savoir ce que fait Extract Method est un prerequis de base. Savoir qu'il *resout* Long Method (SMELL-01), qui *viole* Single Responsibility (LAW-001), qui est *renforce par* le motif Facade (DP-012) — c'est une chaine de raisonnement qu'un LLM ne peut pas construire de maniere fiable par lui-meme. Les 201 relations semantiques de Episteme permettent aux agents IA de traverser ces chemins de maniere deterministe.
</details>

<details>
<summary><b>5. Quand vous prenez des decisions d'architecture et avez besoin de preuves, pas d'opinions</b></summary>

« Dois-je utiliser les microservices ? » — Episteme connecte la question a la Loi de Conway (LAW-017), au SRP (LAW-001) et au motif Strangler Fig (DP-026), puis montre comment ils sont relies. Les decisions deviennent tracables vers des lois d'ingenierie, pas des articles de blog.
</details>

<details>
<summary><b>6. Quand vous avez besoin de conseils d'ingenierie coherents et citables — pas de recommandations hallucinees</b></summary>

Chaque resultat reference des identifiants d'entite explicites (`DP-005`, `RF-001`, `LAW-021`). Les recommandations sont accompagnees de scores de priorite et d'estimations d'effort. La meme requete renvoie toujours la meme reponse structuree.
</details>

<details>
<summary><b>7. Quand vous travaillez dans un environnement deconnecte ou sur un reseau restreint</b></summary>

Episteme fonctionne entierement hors-ligne : binaire unique, base de donnees SQLite locale, embeddings locaux via fastembed (ONNX Runtime). Pas de telemetrie, pas d'appel a un serveur externe, pas d'API externe. Votre code et vos resultats d'analyse ne quittent jamais votre machine.
</details>

---

## Installation

### Option 1 : Une seule commande (Recommandee)

```bash
# La premiere compilation prend 3 a 5 minutes — c'est normal
cargo install --git https://github.com/epicsagas/Episteme
epis install claude    # initialise les donnees + connecte MCP + installe les agents
```

> Apres `epis install claude`, **redemarrez Claude Code** pour que les outils MCP et les agents apparaissent.

### Option 2 : A partir des sources

```bash
git clone https://github.com/epicsagas/Episteme.git
cd Episteme && cargo build --release
```

Puis executez le binaire pour votre plateforme :

| Plateforme | Commande |
|------------|----------|
| **macOS / Linux** | `./target/release/epis install --local claude` |
| **Windows** | `.\target\release\episteme.exe install --local claude` |

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
    "episteme": {
      "command": "docker",
      "args": ["exec", "-i", "episteme-api", "episteme", "mcp"]
    }
  }
}
```

### Option 4 : Binaires precompiles (Rust non requis)

Telechargez le dernier binaire pour votre plateforme depuis [GitHub Releases](https://github.com/epicsagas/Episteme/releases) :

| Plateforme | Fichier |
|------------|---------|
| **macOS** (Apple Silicon) | `episteme-aarch64-apple-darwin.tar.gz` |
| **macOS** (Intel) | `episteme-x86_64-apple-darwin.tar.gz` |
| **Linux** (x86_64) | `episteme-x86_64-unknown-linux-gnu.tar.gz` |
| **Linux** (ARM64) | `episteme-aarch64-unknown-linux-gnu.tar.gz` |
| **Windows** (x86_64) | `episteme-x86_64-pc-windows-msvc.zip` |

```bash
# macOS / Linux
tar xzf episteme-*.tar.gz
sudo mv episteme /usr/local/bin/

# Windows — extrayez le zip et ajoutez episteme.exe a votre PATH
```

Puis installez :
```bash
epis install claude    # ou : cursor, codex, gemini
```

### Verifier

```bash
epis --version
epis stats
epis explore "strategy pattern"    # explorer le graphe de connaissances
```

---

## Outils MCP et Agents

> **Qu'est-ce que MCP ?** Le [Model Context Protocol](https://modelcontextprotocol.io) est une norme ouverte qui permet aux outils IA d'appeler des services externes. Episteme expose son graphe de connaissances sous forme d'outils MCP que Claude Code, Cursor et autres editeurs compatibles peuvent appeler automatiquement.

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
| **`episteme-advisor`** | Decisions d'ingenierie, compromis | Chaines de compromis multi-entites avec plans d'action | code-reviewer, architecture-analyst, researcher |
| **`episteme-researcher`** | Exploration du graphe de connaissances | Cartes de connexions entre motifs, lois, smells | advisor, code-reviewer |
| **`architecture-analyst`** | Evaluation d'architecture par rapport aux lois | Score de conformite avec evaluation ponderee par les risques | advisor, code-reviewer, researcher |

**Exemple de flux de travail** : `code-reviewer` detecte un God Object → trace la causalite vers 3 smells en aval → offre « Appliquer RF-018 » (→ refactoring-expert) ou « Analyse approfondie de la cause racine » (→ episteme-advisor) ou « Verification d'architecture » (→ architecture-analyst).

[Guide complet d'integration MCP](../../docs/mcp-integration-guide.md)

---

## Utilisation CLI

```bash
# Analyser le code pour les smells
epis analyze my_code.py --language python --json
episteme infer my_code.py

# Explorer le graphe de connaissances
epis explore "strategy pattern"
epis graph path DP-005 RF-001   # ex. Factory Method → Extract Method

# Construire l'index RAG
epis build

# Demarrer les serveurs
epis api              # REST API sur :8000
episteme mcp --http       # Serveur MCP sur :43175
episteme web --port 8080  # Interface Web (explorateur de graphe interactif)

# Packaging de distribution
episteme dist --out-dir release/
```

---

## Fonctionnalités

| | Fonctionnalité | Pourquoi c'est important |
|--|----------------|--------------------------|
| 🧠 | **22 motifs de conception GoF** | Catalogue complet avec exemples concrets |
| 🔧 | **66 techniques de refactoring** | Catalogue de Fowler avec exemples de code |
| ⚖️ | **56 lois et principes logiciels** | SOLID, loi de Conway, théorème CAP, etc. |
| 👃 | **17 types de code smells** | Long Method, God Object, Feature Envy, etc. ¹ |
| 🔗 | **201 relations sémantiques** | « résout », « impose », « viole », « est lié à » |
| 🤖 | **6 outils MCP + 4 agents** | Interaction agent IA haute fidélité avec transferts inter-agents |
| 🌍 | **Support de 10 langages** | Python (AST), Java, TypeScript, Go, Rust, C++, C#, PHP, Ruby, Kotlin |
| 📊 | **Analyse déterministe** | Python basé AST + regex multilangage, résultat identique à chaque fois |
| 🏷️ | **Connaissances citables** | Chaque découverte est liée à des IDs d'entité explicites (`RF-001`, `LAW-021`) |
| 🌐 | **API REST (17 points d'accès)** | Auth, limitation de débit, sondes de santé, métriques Prometheus |
| 📦 | **Binaire unique** | Pas de runtime, multiplateforme (macOS, Linux, Windows) |
| 🔌 | **Embeddings locaux** | fastembed (ONNX Runtime), recherche sémantique sans configuration |
| 🐳 | **Support Docker** | Build multi-étape avec vérifications de santé |

> ¹ Duplicate Code (SMELL-13) et Shotgun Surgery (SMELL-09) nécessitent un contexte multi-fichiers et sont ignorés en mode mono-fichier.

---

## Documentation

| Document | Description |
|----------|-------------|
| [Demarrage rapide](../../QUICKSTART.md) | Installation etape par etape, premiere execution, depannage |
| [Guide d'integration MCP](../../docs/mcp-integration-guide.md) | Reference des outils, exemples d'agents, flux de conversation |
| [Reference API](../../docs/api.md) | Points d'acces REST, authentification, exemples |
| [Distribution](../../docs/distribution.md) | Packaging de release et deploiement |
| [Developpement et Contribution](../../DEVELOPMENT.md) | Architecture, comment contribuer |
| [Journal des modifications](../../CHANGELOG.md) | Historique des versions et notes de mise a jour |

---

## Configuration

### Variables d'environnement

```bash
# Emplacements des donnees
EPISTEME_DATA_DIR=~/.episteme/data
EPISTEME_DB_PATH=~/.episteme/db/episteme.db

# Serveur API
EPISTEME_API_HOST=0.0.0.0
EPISTEME_API_PORT=8000
EPISTEME_API_KEY=your-secret-key

# Serveur MCP
EPISTEME_MCP_HOST=127.0.0.1
EPISTEME_MCP_PORT=43175
```

---

## Depannage

**Commande `episteme` introuvable apres l'installation**

| Plateforme | Solution |
|------------|----------|
| **macOS / Linux** | `export PATH="$HOME/.cargo/bin:$PATH"` — ajoutez a `~/.bashrc` ou `~/.zshrc` pour rendre persistant |
| **Windows** | Ajoutez `%USERPROFILE%\.cargo\bin` au PATH systeme, ou ouvrez un nouveau terminal |

**Les outils MCP n'apparaissent pas dans Claude Code / Cursor**

Redemarrez l'editeur apres avoir execute `epis install`. Si toujours absent, verifiez que la configuration a ete ecrite :
```bash
cat ~/.claude.json   # Claude Code
```

**Port deja utilise**
```bash
episteme mcp --http --port 43176   # utiliser un port different
```

**Premier demarrage lent**

Episteme construit un index d'embeddings local au premier lancement. Cela prend 30 a 60 secondes et est un cout ponctuel. Les demarrages suivants sont instantanes.

**Erreurs de compilation pendant `cargo install`**

Assurez-vous que Rust 1.95+ est installe :
```bash
rustup update stable
rustup show   # confirmer la toolchain active
```

> Plus d'aide : [Section depannage de QUICKSTART.md](../../QUICKSTART.md#troubleshooting) · [Ouvrir un ticket](https://github.com/epicsagas/Episteme/issues)

---

## Feuille de route

- [ ] **Entites personnalisees** — Ajouter des motifs/smells specifiques a l'equipe
- [ ] **Tutoriels interactifs** — Visites guidees integrees pour les outils MCP
- [ ] **Metadonnees multilingues** — Titres et resumes des entites en coreen, japonais, chinois (traductions README deja completes)
- [ ] **Descriptions des outils MCP** — Descriptions ameliorees pour remplacer les plugins IDE specifiques
- [ ] **Metriques d'equipe** — Agréger l'utilisation des motifs au sein de l'organisation

---

## Contribuer

Les contributions sont les bienvenues ! Voir [DEVELOPMENT.md](../../DEVELOPMENT.md) pour l'aperçu de l'architecture et le guide de contribution.

```bash
# Executer les tests
cargo test

# Lint
cargo clippy -- -D warnings

# Formatage
cargo fmt
```

Des questions ? [Ouvrir une discussion](https://github.com/epicsagas/Episteme/discussions) ou [creer un ticket](https://github.com/epicsagas/Episteme/issues).

---

## Licence

Apache 2.0 — voir [LICENSE](../../LICENSE) pour plus de details.
