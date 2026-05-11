# Journal des modifications

Tous les changements notables d'Episteme seront documentes dans ce fichier.

Le format est base sur [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
et ce projet adhère a [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Non publie]

### Modifie

- CLI : `explore` renomme en `search` (l'ancien nom fonctionne comme alias deprecie)
- CLI : `mcp` et `api` gerent desormais le cycle de vie complet de leurs services (`start`, `stop`, `restart`, `status`, `enable [--now]`, `disable [--now]`)
- CLI : la commande `service` de premier niveau est depreciee — utilisez `mcp start/stop/restart/status/enable/disable` a la place
- CLI : `mcp --http` est deprecie — utilisez `mcp start` pour le mode daemon HTTP
- CLI : `launchd-install/uninstall/status` est deprecie — utilisez `mcp enable/disable/status` a la place
- `enable/disable` desormais multiplateforme : macOS (launchd) et Linux (unite utilisateur systemd)

### Ajoute

- `api start/stop/restart/status/enable/disable` — gestion du cycle de vie du daemon API REST
- Generation d'unites utilisateur systemd Linux pour `mcp enable`

- **Transport MCP HTTP pour Claude Code** — selecteur de transport TUI, HTTP par defaut, activation automatique launchd
- **Installation automatique des prompts d'agent** — `epis install` copie les prompts d'agent Episteme dans `~/.claude/agents/`
- **Descriptions d'entites** — champ description extrait automatiquement des fichiers source markdown, affiche dans le panneau de details du visualiseur web
- **SPA de visualisation des benchmarks** — analyse de tendances, tableau de bord de decomposition des requetes
- **Refonte du visualiseur web** — mise en page en diagramme Sankey, arbre dans la barre laterale, panneau de details, ameliorations de lisibilite des sous-graphes
- **Upsert de configuration MCP** — relancer `epis install` met a jour le transport si la configuration differe (stdio ↔ HTTP)
- **Configuration MCP YAML** — `mcp.host` / `mcp.port` dans `config.yaml` (yaml → repli sur variable d'environnement)
- **Supervision** — support natif et de scrape a distance pour Prometheus via variables d'environnement
- **Durcissement CI** — cargo audit, gitleaks, generation SBOM, SHA d'actions epinglees
- **Pipeline de release** — cible Windows, publication sur crates.io, tap Homebrew
- **Exemple de diagnostic architectural de module Dieu** dans `examples/`

### Modifie

- **Assistant d'installation** — toutes les etapes (transport, Redis, telemetrie) migrees vers TUI plein ecran
- **Flux d'installation** — construit automatiquement l'index RAG apres le peuplement, ignore si la base existe deja
- **Graphe de connaissances** — enrichi avec des relations semantiques inter-entites
- **Licence** — MIT → Apache-2.0

### Corrige

- Panic du runtime Tokio dans `main()` synchrone pour la telemetrie
- Qualite de recherche — bug de mesure NDCG resolu, precision hit@1 amelioree a 100 %
- Rappel de recherche — boosting inter-types, gestion des entites eparses, synonymes d'intention
- Cache du modele fastembed epingle a `~/.episteme/models`
- Substitution d'UID de bootstrap launchd et gestion du port deja utilise
- Origines CORS desormais configurables via `EPISTEME_CORS_ORIGINS`

## [0.1.0] - 2026-05-03

### Ajoute

- **Reecriture complete en Rust** — remplacement total de la base de code Python par du Rust idiomatique
- **Architecture hexagonale** — `ports/` (traits), `domain/` (logique metier), `adapters/` (infrastructure), `server/` (HTTP)
- **Framework GenericParser** — 8 parseurs bases sur les accolades consolides en `GenericParser` avec `ParserConfig` ; motifs regex mis en cache via `OnceLock` avec `Box::leak`
- **Parsing AST Python** — `rustpython-parser` pour la detection precise des code smells Python (Long Method, Large Class, God Object)
- **TieredAccum + build_detection()** — deduplication de 14 constructions identiques de detection de smells dans `detectors.rs` (1 253 → 591 lignes)
- **Decomposition du module MCP** — separation de `EpistemeMCP` (675 lignes) en services `mcp_search`, `mcp_graph`, `mcp_analysis`
- **Decomposition des commandes CLI** — separation de `main.rs` (1 741 lignes) en module `commands/` avec `cli.rs` pour les definitions clap
- **Deduplication des gestionnaires API** — fusion des doublons `search`/`search_post` en `do_search()` partage
- **16 fonctions de detecteur de smells** — contre 14 precedemment, couvrant toutes les categories de smells GoF
- **17 endpoints API REST** — sondes de sante, metriques Prometheus, CORS, limitation de debit
- **Eviction TTL du limiteur de debit** — MAX_BUCKETS=10 000 avec TTL de 1 heure pour eviter la croissance memoire non bornee
- **Attenuation ReDoS** — regex d'operateur ternaire bornee de `[^:]+` a `[^:\n]{1,50}`
- **Embeddings locaux** — fastembed (ONNX Runtime) pour la recherche semantique sans configuration
- **Assistant d'installation interactif** — TUI avec crossterm, raccourcis vim, ecran alternatif
- **Packaging de distribution** — commande `episteme dist` pour la creation d'archives de release avec amorcage automatique de la base
- **CI multiplateforme** — workflow de release GitHub Actions pour linux/macOS (x86_64 + aarch64)
- **Dockerfile multi-etapes** — compilateur Rust + runtime Debian leger

### Modifie

- **Langage** : Python 3.11+ → Rust (edition 2024)
- **Framework Web** : FastAPI → axum
- **Base de donnees** : Python sqlite3 → rusqlite (integre)
- **Embeddings** : sentence-transformers/PyTorch → fastembed/ONNX Runtime
- **CLI** : argparse → clap (derive)
- **Tous les motifs regex mis en cache** — zero recompilation sur les chemins critiques via le `REGEX_CACHE` global

### Supprime

- Dependance au runtime Python
- Dependance a ChromaDB
- Dependance a tree-sitter
- Workflow de publication PyPI
- Binaire autonome `episteme-hook` (etait un point d'entree PyPI uniquement Python) — utilisez `episteme hooks ground|sniff|audit` a la place

## [0.0.5] - 2026-04-30

### Ajoute

- Interface web de visualisation du graphe (`episteme web`) avec D3-force
- Base vectorielle pre-construite dans l'archive de release
- Drapeau `epis install --local` pour les flux de developpement
- 650+ relations semantiques couvrant les 161 entites
- Generation automatique de la base vectorielle en CI lors des releases

## [0.0.4] - 2026-04-29

### Ajoute

- Serveur MCP avec 6 outils
- 4 agents specialises
- Commande `epis install`
- Gestion de daemon `epis service`
- Recherche hybride (FTS5 + vectorielle)
- Cache Redis, acceleration GPU
- Detection de code smells en 10 langages
- Supervision Prometheus + Grafana
