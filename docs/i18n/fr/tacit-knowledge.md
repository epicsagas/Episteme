# Architecture des connaissances tacites

Episteme gere deux couches distinctes de connaissances : **canonique** (immuable, curee) et **tacite** (mutable, contribuee par les utilisateurs). Ce document decrit l'architecture a deux bases de donnees, le flux de donnees et le cycle de vie des insights.

## Vue d'ensemble

| | Connaissances canoniques | Connaissances tacites (Insights) |
|---|---|---|
| **Stockage** | `~/.episteme/db/episteme.db` | `~/.episteme/user_knowledge.db` |
| **Mutabilite** | Lecture seule (reconstruit via `epis build`) | Lecture-ecriture (temps reel via MCP) |
| **Prefixe d'ID** | `DP-NNN`, `RF-NNN`, `LAW-NNN`, `SMELL-NNN` | `TK-NNN` |
| **Source** | Fichiers markdown cures dans `raw/` | Outil MCP `add_insight` / CLI `epis insight` |
| **Entites** | 22 motifs, 66 refactoring, 56 lois, 23 smells | Insights utilisateurs illimites |

Ces deux bases de donnees sont physiquement separees mais fusionnees au runtime en un seul graphe traversable.

## Conception a deux bases de donnees

```
┌─────────────────────────────────┐     ┌──────────────────────────────┐
│  Base canonique (episteme.db)   │     │  Base de connaissances       │
│                                 │     │  utilisateur                 │
│  ┌───────────┐  ┌────────────┐  │     │  (user_knowledge.db)         │
│  │  chunks   │  │ embeddings │  │     │  ┌────────────────────────┐  │
│  │  (914)    │  │  (914)     │  │     │  │  user_entities         │  │
│  └───────────┘  └────────────┘  │     │  │  (entrees TK-xxx)      │  │
│                                 │     │  ├────────────────────────┤  │
│  Construit par : epis build     │     │  │  user_relations        │  │
│  Peuple depuis : raw/*.md       │     │  ├────────────────────────┤  │
│                                 │     │  │  user_embeddings       │  │
│  Immutable au runtime           │     │  ├────────────────────────┤  │
│                                 │     │  │  user_entities_fts     │  │
└──────────────┬──────────────────┘     │  │  (index de recherche   │  │
               │                        │  │   FTS5)                │  │
               │                        │  ├────────────────────────┤  │
               │                        │  │  insight_seq           │  │
               │                        │  │  (compteur ID atomique)│  │
               │                        │  └────────────────────────┘  │
               │                        │                              │
               │                        │  Ecrit par : MCP add_insight │
               │                        │  Lu par : search_insights    │
               │                        └──────────────┬───────────────┘
               │                                       │
               └───────────────┬───────────────────────┘
                               │
                    ┌──────────▼──────────┐
                    │   CompositeGraph    │
                    │   (fusion en memoire)│
                    │                     │
                    │  - Recherche d'     │
                    │    entite unifiee   │
                    │  - BFS inter-couche │
                    │  - Requetes de      │
                    │    voisins inter-   │
                    │    couches          │
                    │                     │
                    │  Sert toutes les    │
                    │    requetes d'outils│
                    │    MCP              │
                    └─────────────────────┘
```

### Pourquoi des bases de donnees separees ?

1. **Protection** — Les saisies utilisateur ne peuvent pas corrompre les connaissances canoniques curees.
2. **Cycle de vie independant** — Les connaissances canoniques se mettent a jour via le pipeline de build ; les connaissances tacites se mettent a jour en temps reel.
3. **Portabilite** — Partagez `user_knowledge.db` entre machines ou equipes sans toucher a la couche canonique.

## CompositeGraph

La structure `CompositeGraph` (dans `src/domain/composite_graph.rs`) fusionne les deux couches en une seule interface `GraphRepository` au demarrage :

- Charge le `KnowledgeGraph` canonique depuis `relations.json`
- Ouvre `user_knowledge.db` via `UserGraphStore`
- Fournit des methodes `get_entity()`, `get_neighbors()`, `find_path()` unifiees a travers les deux couches
- Les operations utilisateur ne modifient jamais le graphe canonique

### Repli gracieux

Si `user_knowledge.db` ne peut pas etre ouvert (fichier manquant, erreur de permission), le systeme se replie en mode canonique uniquement. Les 6 outils MCP canoniques continuent de fonctionner ; les 3 outils de connaissances tacites retournent une erreur.

## Schema des connaissances utilisateur

```sql
-- Table d'entites principale
CREATE TABLE user_entities (
    id TEXT PRIMARY KEY,                    -- ex : "TK-001"
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    author TEXT NOT NULL DEFAULT 'user',
    confidence REAL NOT NULL DEFAULT 0.5,   -- 0.0 a 1.0
    evidence_count INTEGER NOT NULL DEFAULT 0,
    last_validated TEXT NOT NULL DEFAULT '',
    tags TEXT NOT NULL DEFAULT '[]',        -- tableau JSON
    relations TEXT NOT NULL DEFAULT '{}',   -- JSON : type -> [target_ids]
    created_at TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL DEFAULT '',
    link_provenance TEXT NOT NULL DEFAULT '{}'  -- JSON : entity_id -> metadonnees
);

-- Aretes de relations explicites
CREATE TABLE user_relations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_id TEXT NOT NULL,
    relation_type TEXT NOT NULL,
    to_id TEXT NOT NULL,
    UNIQUE(from_id, relation_type, to_id)
);

-- Vecteurs d'embeddings (f32, boutisme little-endian)
CREATE TABLE user_embeddings (
    entity_id TEXT PRIMARY KEY,
    embedding BLOB NOT NULL
);

-- Index de recherche en texte integral
CREATE VIRTUAL TABLE user_entities_fts USING fts5(
    title, content, tags,
    content=user_entities, content_rowid=rowid
);

-- Sequence ID atomique
CREATE TABLE insight_seq (key TEXT PRIMARY KEY, val INTEGER NOT NULL);
```

## Outils MCP

### add_insight

Cree une entite `TK-NNN` a partir de texte libre. Le systeme automatiquement :

1. **Detecte les liens vers des entites canoniques** — Correspondance de mots-cles en deux phases (filtrage des mots vides + scoring composite) trouve les motifs, lois et smells pertinents.
2. **Verifie les doublons** — Compare avec les insights existants.
3. **Cree des relations `derives_from`** — Pour les liens a haute confiance (score >= 0.5), relie automatiquement aux entites canoniques.
4. **Calcule les correlations** — Trouve des insights lies en utilisant la similarite de Jaccard.

Parametres :
- `text` (requis) — Contenu de l'insight en texte libre
- `project` (optionnel) — Tag de nom de projet
- `tags` (optionnel) — Tags de categorie
- `linked_entities` (optionnel) — IDs d'entites explicites a lier (ex : `["DP-005", "SMELL-01"]`)

### search_insights

Recherche par mots-cles FTS5 dans les insights contribues par les utilisateurs. Retourne les entites `TK-*` correspondantes avec leur contenu et relations.

Parametres :
- `query` (requis) — Requête de recherche en langage naturel
- `limit` (optionnel) — Nombre maximum de resultats (par defaut 10, max 20)

### confirm_links

Valide ou rejette les liens detectes automatiquement entre un insight et des entites canoniques. Chaque confirmation :

- Augmente le score de confiance de l'insight (+0.05 par lien confirme, plafonne a 1.0)
- Enregistre la provenance du lien (source, score, horodatage)
- Prend en charge les relations merge/supersede entre insights

Parametres :
- `insight_id` (requis) — L'ID `TK-NNN`
- `accepted` (requis) — IDs d'entites a confirmer comme liens valides
- `rejected` (optionnel) — IDs d'entites a rejeter
- `merged_with` (optionnel) — ID d'insight cible pour fusion/remplacement

## Cycle de vie d'un insight

```
1. add_insight("마이크로서비스 분리 시 도메인 경계를 먼저 식별하기로 결정")
       │
       ▼
2. Detection automatique de liens : CONWAY-001 (Loi de Conway), DP-026 (Strangler Fig)
       │
       ▼
3. Creation de TK-001 avec derives_from → LAW-017, DP-026
       │
       ▼
4. confirm_links(insight_id="TK-001", accepted=["LAW-017"])
       │
       ▼
5. Confiance augmentee : 0.5 → 0.55
       │
       ▼
6. Plus tard : search_insights("마이크로서비스 분리") → retourne TK-001
       │
       ▼
7. find_path("TK-001", "SMELL-03") → traverse le graphe inter-couches
```

## Types de relations

| Relation | Direction | Description |
|----------|-----------|-------------|
| `derives_from` | TK → Canonique | Insight fonde sur une entite canonique |
| `applies_to` | TK → Canonique | Insight appliquant un motif/loi a un contexte specifique |
| `supersedes` | TK → TK | Un insight plus recent remplace un ancien |
| `related_to` | TK → TK/Canonique | Connexion semantique generale |

## Utilisation en CLI

```bash
# Ajouter un insight
epis insight add "팀에서 God Class 리팩토링 시 Extract Class보다 Facade Pattern이 효과적이었음"

# Rechercher des insights
epis insight search "인증 미들웨어"

# Lister tous les insights
epis insight list
```

## Fichiers sources cles

| Fichier | Role |
|---------|------|
| `src/domain/composite_graph.rs` | Fusion au runtime des couches canonique + utilisateur |
| `src/adapters/user_graph_store.rs` | `MutableGraphRepository` sauvegarde par SQLite |
| `src/server/mcp_insight.rs` | Gestionnaires MCP pour les 3 outils de connaissances tacites |
| `src/adapters/insight_utils.rs` | Generation d'ID, horodatages, utilitaires texte |
| `src/domain/types.rs` | `UserEntity`, `LinkProvenance`, `EntityType::Insight` |
| `src/ports/graph.rs` | Trait `MutableGraphRepository` (14 methodes) |
