# Documentation de l'API REST Episteme

**Version :** 0.1.0
**URL de base :** `http://localhost:8000`

---

## Demarrage rapide

```bash
# Demarrer le serveur
epis api

# Ou avec un hôte/port personnalise
epis api --host 0.0.0.0 --port 8000

# Docker
docker-compose up -d
```

---

## Authentification

Tous les endpoints a l'exception de `/`, `/health`, `/live`, `/ready` necessitent une authentification par cle API.

### Authentification par cle API

**En-tete :** `X-API-Key: <votre-cle-api>`

**Modes :**

1. **Mode Production** - Definir la variable d'environnement `EPISTEME_API_KEYS`
   - Liste de cles API valides separees par des virgules
   - Tous les endpoints proteges necessitent une cle valide
   - Retourne 401 Unauthorized si absent/invalide

2. **Mode Developpement** - Laisser `EPISTEME_API_KEYS` vide ou non defini
   - Aucune authentification requise

### Generer des cles API

```bash
openssl rand -base64 32
```

### Exemples de requetes

```bash
# Avec authentification (production)
curl -X POST http://localhost:8000/analyze \
  -H "Content-Type: application/json" \
  -H "X-API-Key: votre-cle-api" \
  -d '{"code": "def long_method(): pass", "min_confidence": 0.5}'

# Sans authentification (mode dev)
curl -X POST http://localhost:8000/analyze \
  -H "Content-Type: application/json" \
  -d '{"code": "def long_method(): pass"}'
```

---

## Limitation de debit

Tous les endpoints sont soumis a une limitation de debit par adresse IP avec eviction des seaux basee sur TTL.

| Endpoint | Limite de debit | Raison |
|----------|----------------|--------|
| `/analyze` | 20/minute | Intensif en CPU |
| `/refactor` | 20/minute | Intensif en CPU |
| `/search` | 50/minute | Calcul d'embeddings |
| `/stats`, `/graph/*` | 100/minute | Standard |
| `/`, `/health` | Illimite | Public |

En cas de depassement, retourne 429 avec l'en-tete `Retry-After`.

---

## Endpoints

### Sante et informations

#### `GET /`

Informations sur le service.

**Reponse :**
```json
{
  "name": "episteme",
  "version": "0.1.0",
  "description": "Software engineering knowledge graph",
  "endpoints": ["analyze", "search", "graph", "refactor", "stats"]
}
```

#### `GET /health`

Verification de sante avec statut des composants.

**Reponse :**
```json
{
  "status": "healthy",
  "components": {
    "knowledge_graph": "ok",
    "rag_database": "ok",
    "embedding_provider": "local"
  }
}
```

#### `GET /live`

Sonde de vivacite : `{"status": "alive"}`

#### `GET /ready`

Sonde de disponibilite : `{"status": "ready"}` (503 si pas pret)

#### `GET /stats`

Statistiques du graphe.

**Reponse :**
```json
{
  "total_entities": 161,
  "total_edges": 201,
  "by_type": {
    "refactoring": 66,
    "law": 56,
    "pattern": 22,
    "smell": 17
  }
}
```

---

### Analyse de code

#### Code smells pris en charge (16 detecteurs)

| ID | Nom | Langages |
|---|---|---|
| SMELL-01 | Long Method | Tous |
| SMELL-02 | Long Parameter List | Tous |
| SMELL-03 | Primitive Obsession | Python |
| SMELL-04 | Large Class | Tous |
| SMELL-05 | Data Clumps | Tous (stub) |
| SMELL-06 | Switch Statements | Tous |
| SMELL-07 | Data Class | Tous |
| SMELL-09 | Shotgun Surgery | Tous (stub) |
| SMELL-10 | Divergent Change | Tous |
| SMELL-11 | Lazy Class | Tous |
| SMELL-12 | Speculative Generality | Tous |
| SMELL-13 | Duplicate Code | Tous (partiel) |
| SMELL-14 | Middle Man | Tous |
| SMELL-18 | Feature Envy | Tous |
| SMELL-20 | Message Chains | Tous |
| SMELL-21 | God Object | Tous |

#### `POST /analyze`

Detecter les code smells.

**Requete :**
```json
{
  "code": "def long_method():\n    ...",
  "language": "python",
  "min_confidence": 0.5
}
```

**Reponse :**
```json
{
  "count": 2,
  "smells": [
    {
      "smell_id": "SMELL-01",
      "smell_name": "Long Method",
      "confidence": 0.90,
      "location": "temp.py:1",
      "function_name": "long_method",
      "metrics": {
        "loc": 94,
        "cyclomatic_complexity": 27,
        "nesting_depth": 5,
        "parameter_count": 9
      },
      "reasons": ["LOC=94 exceeds 30", "CC=27 exceeds 10"]
    }
  ]
}
```

#### `POST /refactor`

Obtenir des suggestions de refactoring classees pour les smells detectes.

**Requete :**
```json
{
  "code": "def long_method():\n    ...",
  "top_k": 3,
  "min_confidence": 0.5
}
```

**Reponse :**
```json
{
  "count": 1,
  "analyses": [
    {
      "smell": { "smell_id": "SMELL-01", "smell_name": "Long Method" },
      "suggestions": [
        {
          "refactoring_id": "RF-001",
          "title": "Extract Method",
          "priority_score": 0.79,
          "effort": "medium",
          "principles_enforced": ["LAW-040", "LAW-042-S"]
        }
      ]
    }
  ]
}
```

---

### Recherche

#### `GET /search`

Recherche via parametre de requete : `/search?q=strategy+pattern&top_k=5`

#### `POST /search`

Recherche semantique dans la base de connaissances.

**Requete :**
```json
{
  "query": "How to fix Long Method?",
  "top_k": 5,
  "entity_type": "refactoring"
}
```

**Reponse :**
```json
{
  "count": 3,
  "results": [
    {
      "entity_id": "RF-001",
      "title": "Extract Method",
      "category": "refactoring",
      "similarity": 0.85,
      "content": "Extract Method is a refactoring technique..."
    }
  ]
}
```

---

### Graphe de connaissances

#### `GET /graph/{id}`

Obtenir les details d'une entite par son ID.

**Exemple :** `GET /graph/DP-005`

#### `GET /graph/{id}/neighbors`

Obtenir les voisins d'une entite : `/graph/SMELL-01/neighbors?relation_type=solved_by`

#### `POST /graph/neighbors`

Obtenir les voisins (POST).

**Requete :**
```json
{
  "entity_id": "SMELL-01",
  "relation_type": "solved_by"
}
```

#### `GET /graph/path`

Plus court chemin : `/graph/path?from_id=SMELL-01&to_id=LAW-042-S&max_depth=5`

#### `POST /graph/subgraph`

Extraire un sous-graphe.

**Requete :**
```json
{
  "entity_id": "DP-005",
  "depth": 2
}
```

#### `GET /graph/contradictions`

Trouver les entites avec des relations conflictuelles.

#### `POST /graph/infer-transitive`

Inferer les relations d'application transitive.

---

### Supervision

#### `GET /metrics`

Metriques au format Prometheus incluant :
- `http_requests_total` — par methode, endpoint, statut
- `episteme_smells_detected_total` — par smell_id
- `episteme_searches_total` — par entity_type
- `episteme_analysis_duration_seconds` — histogramme

---

## Performance

| Endpoint | Latence moyenne | Notes |
|----------|----------------|-------|
| `/analyze` | ~5ms | Parsing regex + AST (cache OnceLock) |
| `/refactor` | ~10ms | Inclut le parcours de graphe |
| `/search` | ~20ms | FTS5 + similarite cosinus |
| `/graph/neighbors` | ~1ms | Graphe en memoire |
| `/graph/path` | ~5ms | BFS jusqu'a profondeur 5 |

---

## Gestion des erreurs

| Statut | Signification |
|--------|---------------|
| 200 | Succes |
| 400 | Requette incorrecte |
| 401 | Cle API absente/invalide |
| 404 | Entite non trouvee |
| 429 | Limite de debit depassee |
| 500 | Erreur interne |

---

## Variables d'environnement

```bash
# Serveur
EPISTEME_API_HOST=0.0.0.0
EPISTEME_API_PORT=8000
EPISTEME_API_KEYS=key1,key2

# Donnees
EPISTEME_DATA_DIR=~/.episteme/data
EPISTEME_DB_PATH=~/.episteme/db/episteme.db

# Journalisation
RUST_LOG=info
```

---

## Licence

Licence APACHE-2.0
