# Guide de developpement Episteme

**Projet :** Episteme v0.1.0
**Langage :** Rust (edition 2024)
**Dernière mise a jour :** 2026-05-03

---

## Statut actuel

| Composant | Statut | Details |
|-----------|--------|---------|
| **Base de connaissances** | Termine | 22 motifs, 66 refactoring, 56 lois, 23 smells, 201 relations |
| **Detection de code smells** | Production | 16 fonctions de detection, 10 langages |
| **API REST** | Production | 17 endpoints (axum), limitation de debit, authentification |
| **Serveur MCP** | Production | 6 outils, transport stdio + HTTP |
| **Pipeline RAG** | Production | SQLite + FTS5 + fastembed (ONNX) |
| **Visualisation de graphe** | Production | Interface web interactive avec D3-force |

---

## Architecture

Architecture hexagonale (ports et adaptateurs) :

```
src/
├── commands/          # Gestionnaires de sous-commandes CLI (clap)
│   ├── analysis.rs    # analyze, infer
│   ├── build.rs       # build (pipeline RAG)
│   ├── explore.rs     # explore (recherche/REPL)
│   ├── graph.rs       # requetes de graphe
│   ├── install.rs     # assistant d'installation (TUI)
│   ├── service.rs     # gestion du daemon MCP HTTP
│   └── other.rs       # api, mcp, web, telemetrie, hooks
├── adapters/          # Couche infrastructure
│   ├── regex_parsers.rs   # GenericParser (10 langages, cache regex OnceLock)
│   ├── python_ast_parser.rs  # AST Python (rustpython-parser)
│   ├── search_engines.rs  # Mot-cle FTS5 + similarite cosinus
│   ├── service.rs         # Daemon MCP HTTP
│   ├── sqlite_db.rs       # Pool de connexions SQLite
│   ├── cache.rs           # Cache Redis (optionnel)
│   └── ...
├── domain/            # Logique metier (pas de dependances externes)
│   ├── graph.rs       # KnowledgeGraph (BFS, sous-graphe, contradictions, Jaccard)
│   ├── detectors.rs   # 16 detecteurs de smells avec TieredAccum
│   ├── engine.rs      # RefactoringInferenceEngine + RefactoringRanker
│   ├── summarizer.rs  # Optimisation des reponses au niveau detaille
│   └── types.rs       # EntityType, RelationType, types fondamentaux
├── server/            # Couche HTTP (axum)
│   ├── api_routes.rs  # 17 endpoints REST
│   ├── mcp_handler.rs # Façade legère MCP
│   ├── mcp_search.rs  # Service de recherche
│   ├── mcp_graph.rs   # Service de graphe
│   └── mcp_analysis.rs # Service d'analyse de code
└── ports/             # Traits (limites hexagonales)
    ├── parser.rs      # Trait CodeParser
    ├── search.rs      # Trait SearchEngine
    ├── graph.rs       # Trait GraphStore
    └── embeddings.rs  # Trait EmbeddingProvider
```

---

## Pile technologique

| Composant | Technologie | Objectif |
|-----------|------------|---------|
| **Langage** | Rust (edition 2024) | Securite, performance, binaire unique |
| **Framework Web** | axum | API REST + transport MCP HTTP |
| **Base de donnees** | rusqlite (SQLite integre) | Graphe de connaissances + stockage vectoriel |
| **Recherche** | FTS5 + similarite cosinus | Recherche hybride par mots-cles + semantique |
| **Embeddings** | fastembed (ONNX Runtime) | Generation d'embeddings locaux sans configuration |
| **CLI** | clap (derive) | 15 sous-commandes |
| **AST Python** | rustpython-parser | Detection de smells Python basee sur l'AST |
| **Autres langages** | regex (cache OnceLock) | Framework GenericParser |

---

## Detecteurs de code smells (16)

| ID | Smell | Detection |
|----|-------|-----------|
| SMELL-01 | Long Method | Seuil de LOC |
| SMELL-02 | Long Parameter List | Nombre de parametres |
| SMELL-03 | Primitive Obsession | Ratio de parametres primitifs |
| SMELL-04 | Large Class | Nombre de methodes + champs |
| SMELL-05 | Data Clumps | Groupes de parametres repetes (stub) |
| SMELL-06 | Switch Statements | Nombre de switch/match |
| SMELL-07 | Data Class | Ratio methodes vs champs |
| SMELL-08 | Temporary Field | Utilisation conditionnelle de champs (stub) |
| SMELL-09 | Shotgun Surgery | Couplage de changements (stub) |
| SMELL-10 | Divergent Change | Metriques de cohesion des methodes |
| SMELL-11 | Lazy Class | Faible LOC + nombre de methodes |
| SMELL-12 | Speculative Generality | Abstraction sans implementation concrete |
| SMELL-13 | Duplicate Code | Similarite basee sur le hachage (partiel) |
| SMELL-14 | Middle Man | Ratio de delegation |
| SMELL-15 | Parallel Inheritance Hierarchies | Reproduction de hierarchie (stub) |
| SMELL-16 | Comments | Ratio commentaires/code (stub) |
| SMELL-17 | Dead Code | Detection de code inaccessible/inutilise (stub) |
| SMELL-18 | Feature Envy | Ratio d'appels externes |
| SMELL-19 | Inappropriate Intimacy | Acces prive inter-classes (stub) |
| SMELL-20 | Message Chains | Profondeur de chaîne d'appels |
| SMELL-21 | God Object | Composite : LOC + methodes + couplage |
| SMELL-22 | Refused Bequest | Ratio de surcharges vides (stub) |
| SMELL-23 | Alternative Classes with Different Interfaces | Divergence d'interface (stub) |

---

## Configuration du developpement

```bash
# Cloner et compiler (necessite Rust 1.95+)
git clone https://github.com/epicsagas/Episteme.git
cd Episteme
cargo build

# Executer les tests
cargo test

# Linter
cargo clippy -- -D warnings

# Installer localement (peuple les donnees et construit la base automatiquement)
cargo install --path .
epis install --local
```

---

## Endpoints API (17)

| Methode | Chemin | Description |
|---------|--------|-------------|
| GET | `/` | Informations du service |
| GET | `/health` | Verification de sante |
| GET | `/live` | Sonde de vivacite |
| GET | `/ready` | Sonde de disponibilite |
| GET | `/stats` | Statistiques du graphe |
| POST | `/analyze` | Detection de code smells |
| POST | `/refactor` | Suggestions de refactoring |
| GET | `/search` | Recherche dans les connaissances |
| POST | `/search` | Recherche dans les connaissances (POST) |
| GET | `/graph/{id}` | Obtenir une entite |
| GET | `/graph/{id}/neighbors` | Obtenir les voisins |
| POST | `/graph/neighbors` | Obtenir les voisins (POST) |
| POST | `/graph/subgraph` | Extraire un sous-graphe |
| GET | `/graph/path` | Plus court chemin |
| GET | `/graph/contradictions` | Trouver les contradictions |
| POST | `/graph/infer-transitive` | Inferer les relations transitives |
| GET | `/metrics` | Metriques Prometheus |

---

## Feuille de route

- **Plugins IDE** — Integrations natives VSCode, IntelliJ
- **Entites personnalisees** — Ajouter des motifs/smells specifiques a l'equipe
- **Metriques d'equipe** — Agreger l'utilisation des motifs au sein de l'organisation
- **Documentation multilingue** — Base de connaissances en coreen, japonais, chinois
- **Tutoriels interactifs** - Visites guidees integrees pour les outils MCP

---

*Dernière mise a jour : 2026-05-03*
