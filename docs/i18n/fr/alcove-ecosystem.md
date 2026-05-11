# Ecosysteme Alcove — Analyse d'architecture et de capacites

> Une comparaison detaillee de la couche de connaissances tacites d'Episteme (TK-*) et de l'ecosysteme de documentation Alcove, couvrant les modeles de stockage, les capacites de recherche, la gestion du cycle de vie et les recommandations d'utilisation.

---

## 1. Vue d'ensemble de l'architecture

### Connaissances tacites Episteme (TK-*)

| Aspect | Detail |
|--------|--------|
| **Stockage** | Fichier SQLite unique (`~/.episteme/user_knowledge.db`) |
| **Schema** | 5 tables : `user_entities`, `user_relations`, `user_embeddings`, `user_entities_fts` (virtuelle FTS5), `insight_seq` |
| **Unite** | Un insight = une ligne `UserEntity` (ID TK-xxx) |
| **Graphe** | Fusionne avec le graphe canonique via `CompositeGraph` au runtime — permet le parcours de chemin inter-couches (TK-001 → DP-005 → SMELL-01) |
| **Concurrence** | `Mutex<Connection>` + mode WAL pour acces MCP + CLI simultane |

### Systeme de documentation Alcove

| Aspect | Detail |
|--------|--------|
| **Stockage** | Fichiers Markdown sur le systeme de fichiers + index Tantivy BM25 + embeddings sqlite-vec |
| **Structure** | Classification en 3 niveaux : Core (7), Supplementaire (19), Public (15) fichiers par projet |
| **Unite** | Un fichier Markdown structure (PRD, ARCHITECTURE, DECISIONS, etc.) |
| **Graphe** | Connexions lâches basees sur wikiliens et chemins de fichiers |
| **Concurrence** | Verrou base sur fichier (`.index_lock`) par racine de docs, isolation d'index par vault |
| **Vaults** | 3 liens symboliques vers des dossiers Obsidian PARA : areas (8 docs), resources (71), zettelkasten (17) |

---

## 2. Comparaison des modeles de stockage

### Schema TK-* Episteme

```sql
-- Table principale
CREATE TABLE user_entities (
    id TEXT PRIMARY KEY,           -- TK-001, TK-002, ...
    title TEXT,                    -- Auto : premiere ligne, max 80 caracteres
    content TEXT,                  -- Texte libre (pas de longueur max)
    author TEXT DEFAULT 'user',
    confidence REAL DEFAULT 0.5,   -- +0.05 par lien confirme, plafond 1.0
    evidence_count INTEGER DEFAULT 0,
    last_validated TEXT DEFAULT '',
    tags TEXT DEFAULT '[]',        -- Tableau JSON
    relations TEXT DEFAULT '{}',   -- JSON HashMap<relation_type, Vec<entity_id>>
    link_provenance TEXT DEFAULT '{}',
    created_at TEXT,
    updated_at TEXT
);

-- Relations normalisees (derives_from, applies_to, supersedes)
CREATE TABLE user_relations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_id TEXT,
    relation_type TEXT,
    to_id TEXT,
    UNIQUE(from_id, relation_type, to_id)
);

-- Recherche en texte integral FTS5
CREATE VIRTUAL TABLE user_entities_fts USING fts5(title, content, tags, content=user_entities);
```

### Structure de fichiers Alcove

```
~/.alcove/
  config.toml                    # Configuration globale (docs_root, listes de fichiers core/team/public, modele d'embedding)
  docs -> lien symbolique        # → Obsidian/SecondBrain/99-Archives/projects
  vaults/
    areas -> lien symbolique     # → Obsidian/02-Areas (8 docs)
    resources -> lien symbolique # → Obsidian/03-Resources (71 docs)
    zettelkasten -> lien symbolique # → Obsidian/10-Zettelkasten (17 docs)
  models/                        # Modeles d'embedding ONNX mis en cache
  logs/

<docs_root>/<project>/
  .alcove/
    index/                       # Fichiers d'index Tantivy BM25
    index_meta.json              # Empreintes de fichiers (mtime + taille)
    vectors.db                   # Embeddings sqlite-vec
  PRD.md                         # Exigences produit
  ARCHITECTURE.md                # Conception systeme
  PROGRESS.md                    # Jalons et statut
  DECISIONS.md                   # Registre de decisions d'architecture
  CONVENTIONS.md                 # Standards de codage
  SECRETS_MAP.md                 # Variables d'environnement et secrets
  DEBT.md                        # Registre de dette technique
```

---

## 3. Caractere des connaissances

| Dimension | Episteme TK-* | Alcove |
|-----------|---------------|--------|
| **Type** | Insights ponctuels, leçons apprises, decisions d'equipe | Documentation projet structuree (exigences, architecture, decisions) |
| **Mutabilite** | Mutable (CRUD SQLite) | Mutable (editions de fichiers + reconstruction d'index) |
| **Source** | Texte libre contribue par l'utilisateur | Ecrit par l'utilisateur + genere par l'agent depuis des modeles |
| **Autorite** | Observation personnelle/d'equipe | Mandat d'equipe / politique organisationnelle |
| **Granularite** | Atomique (un insight par entree) | Sectionne (plusieurs ADR par DECISIONS.md) |
| **Liaison** | Detectee automatiquement vers des entites canoniques (scoring de mots-cles) | Wikiliens manuels + liens markdown |
| **Versionnage** | Aucun (SQLite uniquement) | Base sur Git (fichier = source de verite) |

### Cycle de vie d'un insight (Episteme TK-*)

```
add_insight(text, tags?, project?, linked_entities?)
  │
  ├── Generer l'ID TK-xxx (sequence atomique)
  ├── detect_canonical_links() — correspondance de mots-cles → top 5 entites canoniques
  │     score >= 0.5 → Lien automatique (derives_from)
  │     score < 0.5 → Lien suggere
  ├── Detection de doublons FTS5 → DuplicateCandidate[]
  ├── Persister dans SQLite + cache en memoire
  └── Retourner : { id, auto_links, suggested_links, duplicates, confidence }

confirm_links(id, accepted[], rejected[])
  │
  ├── Ajouter des relations derives_from/applies_to
  ├── Mettre a jour la source link_provenance a "manual"
  ├── Augmenter la confiance (+0.05 par lien, plafond 1.0)
  └── Persister les mises a jour

search_insights(query, limit?)
  │
  └── Requete FTS5 MATCH → resultats classes
```

### Cycle de vie d'un document (Alcove)

```
init_project(project_name, project_path?)
  │
  ├── Creer 7 documents core depuis les modeles (PRD, ARCHITECTURE, ...)
  ├── Facultativement creer des documents publics (README, CHANGELOG, ...)
  └── Reconstruire l'index de recherche

validate_docs()
  │
  ├── Verifier l'existence des fichiers requis
  ├── Verifier les marqueurs de modeles (TODO, FIXME)
  ├── Verifier les en-tetes de section requis
  ├── Verifier les comptes minimum d'elements de liste
  └── Retourner : succes/avertissement/echec par fichier

lint_project()
  │
  ├── Detecter les [[wikiliens]] et liens markdown casses
  ├── Trouver les fichiers orphelins (non lies depuis aucun document)
  ├── Trouver les marqueurs obsoletes (WIP, TODO, FIXME, DRAFT, DEPRECATED)
  └── Trouver les references d'annees obsoletes (2+ ans)

audit_project()
  │
  ├── Scanner le depot de docs prive pour les documents requis manquants
  ├── Scanner le depot de projet public pour les docs internes exposes
  ├── Classifier les fichiers en niveaux
  └── Retourner : suggested_actions[]
```

---

## 4. Capacites de recherche

| Capacite | Episteme TK-* | Alcove |
|----------|---------------|--------|
| **Moteur** | FTS5 (correspondance de mots-cles) | Tantivy BM25 + similarite cosinus sqlite-vec |
| **Fusion** | Aucune | RRF (Reciprocal Rank Fusion, k=60) |
| **CJK** | Pas de support special | NgramTokenizer (min=2, max=3) |
| **Decoupage** | N/A (une ligne = un insight) | Segments de 200-500 caracteres |
| **Incremental** | N/A (table unique) | Comparaison d'empreintes mtime + taille |
| **Recherche vectorielle** | Le schema existe (`user_embeddings`) mais **non connecte** | Pleinement operationnel (MultilingualE5Small, 384d) |
| **Portee** | Base de donnees unique | Par projet ou globale (inter-projets) |
| **Repli** | Aucun | Correspondance de sous-chaîne grep en l'absence d'index |

---

## 5. Completude des fonctionnalites

| Fonctionnalite | Episteme TK-* | Alcove |
|----------------|---------------|--------|
| Creer | `add_insight` | `init_project`, edition de fichiers |
| Lire | `search_insights` (recherche uniquement, pas d'acces par ID) | `get_doc_file`, `search_project_docs` |
| Mettre a jour | Non expose via MCP | Edition directe de fichier + `rebuild_index` |
| Supprimer | Non expose via MCP | Suppression de fichier + `rebuild_index` |
| Validation | Aucune | `validate_docs`, `lint_project` |
| Audit | Aucun | `audit_project` (separation public/prive) |
| Sauvegarde | Aucune | `backup_vault` (instantane de commit git) |
| Import | Aucun | `promote_document` (Obsidian → depot de docs) |
| Politique | Aucune | `policy.toml` avec niveaux d'application |
| Modeles | Aucun | 7 core + 19 supplementaires + 15 publics |

---

## 6. Systeme de vaults Alcove

Trois vaults, lies symboliquement a la structure PARA d'Obsidian :

| Vault | Cible | Documents | Objectif |
|-------|-------|-----------|----------|
| `areas` | `02-Areas` | 8 | Domaines : agents MCP, DevOps, Rust, LLM/RAG, Open Source |
| `resources` | `03-Resources` | 71 | Reference : AWS, Lois de l'ingenierie logicielle, docs techniques |
| `zettelkasten` | `10-Zettelkasten` | 17 | Notes atomiques : architecture IA, BM25, graphes de connaissances, motifs Rust |

Chaque vault possede de maniere independante :
- Index BM25 (Tantivy)
- Base de donnees vectorielle (sqlite-vec)
- Suivi des empreintes de fichiers (`index_meta.json`)
- Isolation du cache (separes `OnceLock<Mutex<HashMap>>`)

---

## 7. Systeme de configuration Alcove

### Global : `~/.alcove/config.toml`

```toml
docs_root = "/chemin/vers/Obsidian/SecondBrain/99-Archives/projects"

[core]
files = ["PRD.md", "ARCHITECTURE.md", "PROGRESS.md", "DECISIONS.md",
         "CONVENTIONS.md", "SECRETS_MAP.md", "DEBT.md"]

[team]
files = ["ENV_SETUP.md", "ONBOARDING.md", "DATA_MODEL.md", "SCHEMA.md",
         "DEPLOYMENT.md", "RUNBOOK.md", "PLAYBOOK.md", "MONITORING.md", ...]  # 19 fichiers

[public]
files = ["README.md", "CHANGELOG.md", "CONTRIBUTING.md", "SECURITY.md", ...]  # 15 fichiers

[embedding]
model = "MultilingualE5Small"
auto_download = true
enabled = true
```

### Par projet : `alcove.toml`

Remplace les valeurs par defaut globales pour : `diagram_format`, `core_files`, `team_files`, `public_files`.

### Politique : `policy.toml`

Definit :
- Niveau `enforce` : `strict` | `warn` | `off`
- Documents requis avec en-tetes de section et comptes minimum d'elements
- Conventions de nommage (`UPPER_SNAKE`, `lower_snake`, `kebab`, `free`)
- Priorite : projet > equipe > valeurs par defaut integrees

---

## 8. Matrice de decision par cas d'usage

| Situation | Outil recommande | Justification |
|-----------|------------------|---------------|
| « Enregistrer une leçon apprise d'un incident en production » | **Episteme TK-*** | Lie automatiquement aux smells/lois pertinents pour un futur recoupement |
| « Demarrer la documentation pour un nouveau projet » | **Alcove** `init_project` | 7 modeles core generes automatiquement |
| « Verifier si des docs sont obsoletes » | **Alcove** `lint_project` | Detecte automatiquement WIP/TODO/DEPRECATED/dates obsoletes |
| « Trouver ce que l'equipe a decide a propos du middleware d'auth » | **Alcove** `search_project_docs` | Recherche dans le DECISIONS.md structure avec BM25 + vectoriel |
| « Detecter les code smells dans un module » | **Episteme** `analyze_code` | Detection de smells basee sur motifs/regex |
| « S'assurer que le PRD a toutes les sections requises » | **Alcove** `validate_docs` | Validation de sections et comptes d'elements basee sur la politique |
| « Lier un insight au Pattern Strategy » | **Episteme** `confirm_links` | Cree une arete `derives_from` vers l'entite canonique |
| « Importer des notes Obsidian pour l'acces agent » | **Alcove** `promote_document` | Importe dans le depot de docs avec detection automatique de projet |
| « Trouver la relation entre SRP et Extract Class » | **Episteme** `find_path` | Traversal de graphe multi-sauts a travers les types d'entites |
| « Sauvegarder l'etat de la documentation projet » | **Alcove** `backup_vault` | Instantane de commit git avec horodatage |
| « Auditer les docs internes exposes dans le depot public » | **Alcove** `audit_project` | Scanne les emplacements prives et publics |
| « Obtenir des suggestions de refactoring classees pour le code » | **Episteme** `suggest_refactorings` | Scoring composite : severite × effort × alignement des principes |

---

## 9. Roles complementaires

```
Episteme TK-*                     Alcove
"Quel principe universel          "Qu'a decide notre equipe
 s'applique ici ?"                 a ce sujet ?"

 Insight ponctuel ←─────────────→ Enregistrement de decision structure
 Lien automatique par mots-cles    Echafaudage base sur des modeles
 Traversal de graphe inter-couches Recherche de documents inter-projets
 Analyse de code → detection de    Analyse de docs → detection d'obsolescence
   smells
```

**Quand les deux sont actifs** : Episteme fournit le « pourquoi » universel (lois, motifs), Alcove fournit le « ce que nous avons decide » specifique au projet (ADR, conventions). Les agents doivent citer les deux sources, Alcove prenant la precedence lorsque les regles d'equipe sont en conflit avec les recommandations generiques.

---

## 10. Echelle et performance

| Metrique | Episteme TK-* | Alcove |
|----------|---------------|--------|
| **Capacite concue** | Des centaines d'insights | ~10 000 fichiers |
| **Latence de recherche** | FTS5 instantane (en memoire) | BM25 < 500ms pour l'apercu |
| **Efficacite en tokens** | Un insight par resultat | Top-5 segments ~1.5k tokens (vs ~8k pour grep) |
| **Reconstruction d'index** | Pas necessaire (declencheurs FTS5) | Incremental : uniquement les fichiers modifies |
| **Taille du modele** | N/A (non connecte) | 15Mo (ArcticEmbedXS) a 2.3Go (BGE-M3) |

---

*Voir aussi : [Guide d'integration Alcove](./alcove-integration.md) pour les motifs d'utilisation et exemples de flux de travail.*
