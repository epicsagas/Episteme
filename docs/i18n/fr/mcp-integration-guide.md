# Guide d'integration MCP

> Integrez le graphe de connaissances d'Episteme dans Claude Code, Cursor et d'autres outils IA compatibles MCP

## Mode MCP HTTP Rust (actuel)
Utilisez le transport HTTP autonome directement :

```bash
# Demarrer MCP via HTTP
episteme mcp --http --host 127.0.0.1 --port 43175
```

Comportement d'authentification :
- Si `EPISTEME_API_KEYS` est configure, les requetes doivent inclure :
```http
Authorization: Bearer <cle-api>
```
- Si aucune cle n'est configuree, l'authentification est ignoree (mode developpement).
- `GET /health` est toujours public pour les verifications de sante.

Note :
- `epis service` gere ce même mode MCP HTTP en arriere-plan (`start|stop|status|enable|disable`).
- Les anciens exemples `--proxy` sont deprecies ; utilisez `mcp --http`/`service` directement.

## Qu'est-ce que MCP ?

[Model Context Protocol (MCP)](https://modelcontextprotocol.io) est une norme ouverte qui permet aux assistants IA d'acceder a des outils externes et des sources de donnees. Episteme fournit 6 outils MCP qui donnent aux agents IA un acces direct aux connaissances en ingenierie logicielle.

---

## Demarrage rapide (Claude Code)

### 1. Installer Episteme

```bash
# Installer (necessite Rust 1.95+)
cargo install --git https://github.com/epicsagas/Episteme

# Installer les agents et le serveur MCP dans Claude Code
# (peuple les donnees et configure MCP automatiquement)
epis install claude
```

> Si le telechargement des donnees echoue, utilisez l'installation depuis les sources : `git clone` → `cargo build --release` → `epis install --local`

### 2. Verifier l'installation

Verifiez `~/.claude/claude_desktop_config.json` :

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

### 3. Commencer a utiliser

Redemarrez Claude Code. Vous avez maintenant acces a 6 outils Episteme :

```
Utilisateur : "Quelle est la meilleure façon de corriger un smell God Object ?"

Claude (utilisant l'outil search_knowledge) :
  → Recherche les refactoring pour "God Object"
  → Retourne : RF-018 (Extract Class), RF-023 (Move Method)

Claude : "L'anti-pattern God Object (SMELL-03) viole le Principe de
Responsabilite Unique (LAW-001). Meilleurs refactoring :

1. Extract Class (RF-018) - Deplacer les methodes/champs associes vers une nouvelle classe
2. Move Method (RF-023) - Relocaliser les methodes dans les classes appropriees

Les deux appliquent les principes SOLID et ameliorent la testabilite."
```

---

## Reference des outils MCP

### 1. `search_knowledge`

**Objectif** : Recherche semantique sur toutes les entites (motifs, lois, refactoring, smells)

**Parametres** :
```typescript
{
  query: string          // Requête en langage naturel
  top_k?: number         // Resultats a retourner (par defaut : 5)
  filter_type?: string   // "pattern", "law", "refactoring", "smell"
}
```

**Retourne** :
```typescript
{
  results: [{
    entity_id: string     // ex : "DP-023"
    title: string
    entity_type: string
    similarity: number    // 0.0-1.0
    summary: string
  }]
}
```

**Exemple de conversation** :
```
Utilisateur : "Comment rendre mon code plus testable ?"

Claude appelle : search_knowledge({
  query: "improve testability",
  top_k: 3
})

Retourne :
- LAW-001: Single Responsibility Principle
- DP-018: Dependency Injection
- RF-042: Extract Interface

Claude : "Trois approches cles pour ameliorer la testabilite :
1. Appliquer SRP (LAW-001) - Une classe, une raison de changer
2. Utiliser l'Injection de Dependances (DP-023) - Injecter les dependances
3. Extraire l'Interface (RF-042) - Simuler les dependances externes"
```

---

### 2. `get_entity`

**Objectif** : Obtenir les details complets d'une entite specifique par son ID

**Parametres** :
```typescript
{
  entity_id: string   // ex : "DP-023", "RF-001", "SMELL-01"
}
```

**Retourne** :
```typescript
{
  entity_id: string
  title: string
  type: string
  description: string
  implementation: string    // Exemples de code
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

**Exemple de conversation** :
```
Utilisateur : "Expliquez le Pattern Strategy en detail"

Claude appelle : get_entity({ entity_id: "DP-023" })

Retourne :
- Description complete du Pattern Strategy
- Quand l'utiliser (algorithmes multiples, commutation au runtime)
- Implementation (Context, interface Strategy, ConcreteStrategies)
- Avantages : Principe Open/Closed, flexibilite au runtime
- Compromis : Augmentation du nombre de classes
- Lie : LAW-002 (Open/Closed), RF-034 (Replace Conditional with Polymorphism)

Claude : "Le Pattern Strategy (DP-023) encapsule les algorithmes dans des classes separees...
[fournit une explication detaillee avec un exemple de code]

Concepts lies :
- Applique le Principe Open/Closed (LAW-002)
- Souvent applique via Replace Conditional with Polymorphism (RF-034)"
```

---

### 3. `get_neighbors`

**Objectif** : Explorer les entites connectees a une entite donnee

**Parametres** :
```typescript
{
  entity_id: string
  relation_type?: string   // "solves", "enforces", "violates", "relates_to"
  max_depth?: number       // Par defaut : 1
}
```

**Retourne** :
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

**Exemple de conversation** :
```
Utilisateur : "Quels refactoring resolvent le smell Long Method ?"

Claude appelle : get_neighbors({
  entity_id: "SMELL-01",
  relation_type: "solved_by"
})

Retourne :
- RF-001: Extract Method
- RF-034: Decompose Conditional
- RF-056: Replace Temp with Query

Claude : "Long Method (SMELL-01) peut etre resolu avec :
1. Extract Method (RF-001) - Decomposer en methodes plus petites
2. Decompose Conditional (RF-034) - Simplifier les conditions complexes
3. Replace Temp with Query (RF-056) - Eliminer les variables temporaires"
```

---

### 4. `find_path`

**Objectif** : Trouver une connexion entre deux entites (ex : comment une loi se rapporte a un motif)

**Parametres** :
```typescript
{
  from_id: string
  to_id: string
  max_depth?: number   // Par defaut : 5
}
```

**Retourne** :
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

**Exemple de conversation** :
```
Utilisateur : "Comment le Principe de Responsabilite Unique se rapporte-t-il au refactoring Extract Class ?"

Claude appelle : find_path({
  from_id: "LAW-001",
  to_id: "RF-018"
})

Retourne le chemin :
LAW-001 (SRP) --enforces--> DP-007 (Facade) --achieved_via--> RF-018 (Extract Class)

Claude : "SRP (LAW-001) se connecte a Extract Class (RF-018) en 2 sauts :
1. SRP applique le Pattern Facade (DP-007)
2. Facade est souvent realise via Extract Class (RF-018)

Insight : Lorsqu'une classe a plusieurs responsabilites, extraire
des classes (RF-018) aide a satisfaire SRP (LAW-001)."
```

---

### 5. `analyze_code`

**Objectif** : Detecter les code smells via analyse AST

**Parametres** :
```typescript
{
  code: string
  language: string   // "python", "java", "typescript", etc.
  min_confidence?: number   // Par defaut : 0.5
}
```

**Retourne** :
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

**Exemple de conversation** :
```
Utilisateur : "Revoyez ce code de validation de paiement"
[colle le code]

Claude appelle : analyze_code({
  code: "...",
  language: "python"
})

Retourne :
- SMELL-01: Long Method (confiance 0.87, LOC=45)
- SMELL-08: Long Parameter List (confiance 0.92, params=9)

Claude : "J'ai trouve 2 code smells :

1. Long Method (SMELL-01, confiance 87 %)
   - 45 lignes de code (seuil : 20)
   - Complexite cyclomatique elevee (12)
   - Recommandation : Extract Method (RF-001)

2. Long Parameter List (SMELL-08, confiance 92 %)
   - 9 parametres (seuil : 4)
   - Recommandation : Introduce Parameter Object (RF-029)"
```

---

### 6. `suggest_refactorings`

**Objectif** : Obtenir des suggestions de refactoring classees pour les smells detectes

**Parametres** :
```typescript
{
  code: string
  language: string
  top_k?: number   // Nombre de suggestions (par defaut : 3)
  min_confidence?: number
}
```

**Retourne** :
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

**Exemple de conversation** :
```
Utilisateur : "Que devrais-je refactoring dans ce code ?"
[colle une grande classe]

Claude appelle : suggest_refactorings({
  code: "...",
  language: "java",
  top_k: 3
})

Retourne les suggestions classees pour le smell God Object :
1. RF-018: Extract Class (priorite 0.89, Severite Elevee, Effort Moyen)
2. RF-023: Move Method (priorite 0.76, Severite Moyenne, Effort Faible)
3. RF-042: Extract Interface (priorite 0.65, Severite Faible, Effort Faible)

Claude : "Ce code presente des caracteristiques de God Object (SMELL-03).

Priorites de refactoring :

1. Extract Class (RF-018) - Priorite 0.89
   - Severite : Elevee
   - Effort : Moyen
   - Deplacer les methodes liees au paiement vers la classe PaymentProcessor
   - Applique : SRP (LAW-001), SoC (LAW-012)

2. Move Method (RF-023) - Priorite 0.76
   - Severite : Moyenne
   - Effort : Faible
   - Relocaliser les methodes de validation vers la classe Validator
   - Victoire rapide pour ameliorer la cohesion

Commencez par #2 (Move Method) pour une amelioration immediate,
puis attaquez #1 (Extract Class) pour le long terme."
```

---

## Agents IA

Episteme inclut 4 agents specialises qui travaillent ensemble comme un systeme connecte. Chaque agent utilise les outils du graphe de connaissances pour l'analyse et presente des options de suivi interactives qui peuvent etre transferees a d'autres agents.

### Reseau d'agents

```
code-reviewer ←→ episteme-advisor
      ↕                ↕
architecture-analyst ←→ episteme-researcher
      ↕
refactoring-expert (externe)
```

Chaque agent termine son rapport par des **Prochaines etapes** — des options interactives qui se connectent a d'autres agents ou outils. Cela cree des flux de bout en bout de la detection a la remediation.

---

### 1. `code-reviewer`

**Quand l'utiliser** : Revue de code pour les smells, violations SOLID et priorites de refactoring

**Ce qu'il fait differemment** : Va au-dela de la detection — trace les chaînes de causalite entre les smells (quels smells sont des causes racines vs. des symptomes derives), valide les resultats par rapport aux conventions de l'ecosysteme du langage (Rust, Go, etc.) et calibre la severite de maniere contextuelle.

**Outils utilises** : `analyze_code` → `suggest_refactorings` → `get_entity` → `get_neighbors` → `find_path`

**Exemple de prompt** :
```
"Revoyez ce module d'authentification"
[colle le code]
```

**Reponse de l'agent** :
```
# Revue de code Episteme

## Resume
AuthenticationModule presente un smell racine God Object causant 3
problemes derives. Corriger la cause racine provoquera des ameliorations en cascade.

## Detections de smells
| Smell | Emplacement | Severite | Confiance | Principe lie |
|-------|-------------|----------|-----------|--------------|
| God Object (SMELL-03) | auth.rs:1-420 | Critique | confirme | LAW-001 (SRP) |
| Long Method (SMELL-01) | auth.rs:88-142 | Elevee | confirme | LAW-001 (SRP) |
| Divergent Change (SMELL-09) | auth.rs:200-380 | Moyenne | confirme | LAW-012 (SoC) |
| Feature Envy (SMELL-15) | auth.rs:290-333 | Moyenne | confirme | LAW-003 (LSP) |

## Analyse de causalite
Causes racines :
- SMELL-03 → provoque → SMELL-01, SMELL-09
  (corriger God Object provoquera des ameliorations en cascade)

Independant :
- SMELL-15 — autonome, aucun impact derive

## Refactoring classes
1. **Extract Class** (RF-018) — Priorite : Critique
   - Adresse : SMELL-03 (cause racine)
   - Effet cascade : resout egalement SMELL-01, SMELL-09
   - Que faire : Extraire les classes SessionManager, EmailVerifier

## Prochaines etapes
1. Refactoring — Appliquer RF-018 (Extract Class) → utiliser l'agent refactoring-expert
2. Approfondissement — Explorer la chaîne de cause racine pour SMELL-03 → utiliser episteme-advisor
3. Verification d'architecture — Evaluer l'impact structurel → utiliser architecture-analyst
4. En savoir plus — Obtenir l'explication complete de SMELL-03 → get_entity(detail_level="full")
```

**Validation** : Avant de confirmer un smell, l'agent verifie croise avec les idiomismes du langage (ex : les grandes structs sont normales en Rust), le contexte de domaine (fichiers de test, code genere) et l'impact reel (cosmetique vs. bloquant pour le developpement).

---

### 2. `episteme-advisor`

**Quand l'utiliser** : Prendre des decisions d'ingenierie (selection de motifs, compromis architecturaux, analyses approfondies des causes racines)

**Outils utilises** : `search_knowledge` → `get_entity` → `get_neighbors` → `find_path`

**Exemple de prompt** :
```
"Devrais-je utiliser un Singleton pour ma classe ConfigManager ?"
```

**Reponse de l'agent** :
```
# Conseil : Singleton vs DI pour ConfigManager

## Entites applicables
### 1. Pattern Singleton (DP-006) - Primaire
- **Ce qu'il dit** : Garantir qu'une classe n'a qu'une seule instance
- **Ce que cela signifie pour vous** : Acces global a la config mais couplage cache
- **Recommandation** : Eviter — viole la testabilite

### 2. Injection de Dependances (DP-018) - Alternative
- **Ce qu'il dit** : Injecter les dependances via le constructeur
- **Ce que cela signifie pour vous** : Dependances explicites, simulation facile
- **Recommandation** : Preferer pour ConfigManager

## Compromis
- Singleton (DP-006) offre de la commodite mais viole DI (DP-018)
- DI ajoute du boilerplate mais permet la conformite LAW-005 (DIP)
- Resolution : Utiliser DI avec une interface de fournisseur de config

## Plan d'action
1. Immediate : Injecter ConfigManager via le constructeur
2. Court terme : Extraire l'interface ConfigProvider
3. Long terme : Considerer des strategies de config par environnement

## Prochaines etapes
1. Implementer — Appliquer le motif DI → utiliser code-reviewer pour valider
2. Explorer les alternatives — Comparer les approches → utiliser episteme-researcher
3. Impact architectural — Evaluer les effets derives → utiliser architecture-analyst
```

---

### 3. `episteme-researcher`

**Quand l'utiliser** : Explorer les relations dans le graphe de connaissances, trouver des alternatives

**Outils utilises** : `search_knowledge` → `get_entity` → `get_neighbors` → `find_path`

**Exemple de prompt** :
```
"Quelles lois le Pattern Observer applique-t-il ?"
```

**Reponse de l'agent** :
```
# Rapport de recherche : Lois appliquees par le Pattern Observer

## Entites primaires
1. **Pattern Observer** (DP-012) - Comportemental
   - Definition : Definir une dependance un-vers-plusieurs entre des objets
   - Quand le changement d'un objet modifie automatiquement d'autres

## Carte de connexions
DP-012 --enforces--> LAW-002 (Open/Closed)
DP-012 --enforces--> LAW-005 (Dependency Inversion)
DP-012 --enforces--> LAW-001 (Single Responsibility)

## Contradictions et compromis
- Observer (DP-012) permet le couplage lache mais peut causer des chaînes
  de mises a jour inattendues — contraste avec Mediator (DP-014) qui centralise

## Prochaines etapes
1. Obtenir des conseils — Transformer les resultats en recommandations → utiliser episteme-advisor
2. Verifier le code — Detecter Observer dans votre base de code → utiliser code-reviewer
3. Evaluer l'architecture — Evaluer l'adequation structurelle → utiliser architecture-analyst
```

---

### 4. `architecture-analyst`

**Quand l'utiliser** : Evaluer l'architecture systeme par rapport aux lois, motifs et risques structurels

**Outils utilises** : `search_knowledge` → `get_entity` → `get_neighbors` → `find_path`

**Exemple de prompt** :
```
"Cette architecture microservices suit-elle la Loi de Conway ?"
[decrit l'architecture]
```

**Reponse de l'agent** :
```
# Analyse architecturale : Microservices

## Conformite aux lois et principes
| Principe | Statut | Preuve | Impact |
|-----------|--------|--------|--------|
| Loi de Conway (LAW-042) | violee | Shipping s'etend sur 2 equipes | Surcout de coordination |
| SRP (LAW-001) | a risque | Analytics depend de tout | Couplage serre |
| Bounded Context (LAW-031) | viole | Pas de limites de domaine claires | Confusion de donnees partagees |

## Tensions cles
- La Loi de Conway (LAW-042) requiert un alignement equipe↔service
  mais le service Shipping s'etend sur les equipes Commerce + Platform
- Trace via : LAW-042 → related_to → LAW-001 → enforced_by → DP-026 (Strangler Fig)

## Recommandations architecturales
1. **Critique** : Deplacer Shipping vers l'equipe Commerce — LAW-042 predit un echec de coordination
2. **Elevee** : Introduire un Event Bus pour Analytics — decoupler via evenements asynchrones
3. **Moyenne** : Definir les Bounded Contexts — aligner les limites de service avec le domaine

## Scores de conformite
- Global : 5/10 | Structure : 4/10 | Evolutivite : 6/10 | Maintenabilite : 5/10

## Prochaines etapes
1. Obtenir des conseils — Resoudre les tensions cles → utiliser episteme-advisor
2. Verifier le code — Detecter les smells structurels → utiliser code-reviewer
3. Rechercher des alternatives — Trouver de meilleurs motifs → utiliser episteme-researcher
```

---

## Chaînes de flux de travail

Les agents et outils se connectent en pipelines de bout en bout. Chaque chaîne produit un rapport suivi d'options de suivi interactives.

### Chaîne 1 : Pipeline de revue de code
```
analyze_code → suggest_refactorings → get_neighbors("solved_by")
  → find_path(smell_A, smell_B) → Rapport avec graphe de causalite
  → L'utilisateur choisit : Appliquer la correction / Approfondir / Verification d'architecture / En savoir plus
```

### Chaîne 2 : Pipeline de revue d'architecture
```
search_knowledge → get_entity → get_neighbors("enforces")
  → get_neighbors("violates") → find_path → Rapport de conformite
  → L'utilisateur choisit : Plan de refactoring / Conseil / Rechercher des alternatives
```

### Chaîne 3 : Pipeline de diagnostic de probleme
```
search_knowledge(symptomes) → get_entity → get_neighbors("solved_by")
  → Rapport de cause racine → L'utilisateur choisit : Appliquer la correction / Conseil / Verifier
```

### Chaîne 4 : Pipeline d'apprentissage
```
search_knowledge(sujet) → get_entity → get_neighbors("related_to")
  → Carte conceptuelle → L'utilisateur choisit : Exemples de code / Appliquer au code / Comparer
```

### Regles de chaînage inter-outils

Chaque appel d'outil mene naturellement au suivant :

| Apres avoir appele... | Toujours suivre avec... |
|------------------------|------------------------|
| `analyze_code` | `suggest_refactorings` sur les smells detectes |
| `suggest_refactorings` | `get_neighbors(smell_id, "solved_by")` pour les alternatives |
| `search_knowledge` | `get_entity` sur les 1-2 meilleurs resultats |
| `get_entity` (smell) | `get_neighbors(id, "violates")` pour les principes impactes |
| `get_entity` (motif) | `get_neighbors(id, "enforces")` pour les lois appliquees |
| Plusieurs smells detectes | `find_path(smell_A, smell_B)` pour la cartographie de causalite |

---

## Installation pour d'autres outils

### Cursor

```bash
epis install cursor
```

Ajoute la configuration MCP a `~/.cursor/mcp.json` :
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

Genere `AGENTS.md` a la racine du projet avec les definitions d'agents.

### Integration MCP personnalisee

Si votre outil prend en charge MCP, configurez manuellement :

```json
{
  "mcpServers": {
    "episteme": {
      "command": "/chemin/vers/episteme",
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

## Execution en tant que service en arriere-plan

Pour de meilleures performances, executez Episteme MCP comme proxy HTTP persistant :

```bash
# Demarrer le service en arriere-plan
epis service start

# Verifier le statut
epis service status
# Sortie : Running on http://localhost:43175 (PID 12345)

# Activer le demarrage automatique au boot (macOS)
epis service enable

# Arreter le service
epis service stop
```

Mettre a jour la configuration MCP pour utiliser le proxy HTTP :

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

Journaux : `~/.episteme/logs/mcp.out.log`

---

## Depannage

### Les outils n'apparaissent pas dans Claude

1. Verifiez que le fichier de configuration existe : `cat ~/.claude/claude_desktop_config.json`
2. Verifiez que episteme est dans le PATH : `which episteme`
3. Testez MCP directement : `episteme mcp`
4. Consultez les journaux : `tail -f ~/.episteme/logs/mcp.err.log`

### Erreur « Base de donnees introuvable »

```bash
# Reconstruire la base de connaissances
epis build --rebuild
```

### Reponses de recherche lentes

```bash
# Utiliser l'acceleration GPU
epis build --gpu

# Ou executer en tant que service en arriere-plan (chauffe plus rapide)
epis service start
```

### L'agent n'utilise pas les outils

Assurez-vous que l'agent a la capacite d'appeler des outils. Dans Claude Code :
```
Utilisateur : "Utilise Episteme pour trouver des motifs pour la logique de retry"
              ^^^^ mentionner explicitement l'utilisation de l'outil
```

---

## Avance : Integration de connaissances personnalisees

Combiner Episteme (connaissances generiques) avec Alcove (connaissances d'equipe) :

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

Voir le [Guide d'integration Alcove](./alcove-integration.md) pour les motifs a double source.

---

## Alternative API

Si votre outil IA ne prend pas en charge MCP, utilisez l'API REST :

```bash
# Demarrer le serveur API
docker-compose up -d

# Utiliser depuis n'importe quel outil
curl http://localhost:8000/search?q=strategy+pattern
```

Voir la [Documentation API](./api.md) pour les endpoints.

---

## Declenchement automatique (Claude Code)

Lorsque vous decrivez un probleme en langage naturel, Claude Code detecte automatiquement l'intention et appelle l'outil Episteme approprie — **vous n'avez pas besoin de mentionner Episteme explicitement**. Ci-dessous, les motifs de declenchement exacts et des exemples.

### Comment ça fonctionne

```
Votre saisie en langage naturel
    ↓ Claude detecte les mots-cles/motifs
    ↓ L'outil Episteme est appele automatiquement
    ↓ Le graphe de connaissances retourne des donnees verifiees
    ↓ (Motifs de conception · Code Smells · Techniques de Refactoring · Lois d'ingenierie)
    ↓ La reponse de Claude est fondee sur des preuves
```

> **Note :** Il s'agit d'un declenchement automatique base sur les prompts, pas d'un hook strict. Pour garantir un appel, utilisez directement le skill `/episteme`.

### Problemes de structure de code

| Ce que vous dites (exemples) | Ce qu'Episteme detecte | Appel d'outil automatique |
|-------------------------------|------------------------|--------------------------|
| "Cette classe fait trop de choses", "Ce fichier depasse 300 lignes" | God Class, Large Class, Single Responsibility | `search_knowledge("god class large class single responsibility")` |
| "Cette fonction est trop longue", "Trop de lignes dans cette methode" | Long Method | `search_knowledge("long method extract method")` |
| "Le code est trop complexe", "Difficile a suivre" | Complexity, Cognitive Overload | `search_knowledge("complexity smell cognitive overload")` |
| "J'ai copie-colle ça partout", "Il y a de la logique dupliquee" | Duplicated Code, Clone | `search_knowledge("duplicated code clone smell")` |

### Problemes de couplage et dependances

| Ce que vous dites (exemples) | Ce qu'Episteme detecte | Appel d'outil automatique |
|-------------------------------|------------------------|--------------------------|
| "La logique metier appelle la DB directement" | Coupling, Persistence, Repository | `search_knowledge("coupling persistence repository data access layer")` |
| "Changer X casse Y", "Les changements se propagent partout" | Brittle Coupling, Change Propagation | `search_knowledge("brittle coupling change propagation rigidity")` |
| "Ajouter un nouveau type oblige a toucher partout", "le switch-case ne cesse de grandir" | Open/Closed, Strategy, Polymorphism | `search_knowledge("open closed principle strategy polymorphism")` |

### Problemes de tests et qualite

| Ce que vous dites (exemples) | Ce qu'Episteme detecte | Appel d'outil automatique |
|-------------------------------|------------------------|--------------------------|
| "C'est difficile a tester", "Je ne peux pas ecrire de tests unitaires pour ça" | Testability, Dependency Injection | `search_knowledge("testability dependency injection mockability")` |

### Problemes de performance et concurrence

| Ce que vous dites (exemples) | Ce qu'Episteme detecte | Appel d'outil automatique |
|-------------------------------|------------------------|--------------------------|
| "L'API est lente", "Le temps de reponse est trop eleve" | N+1 Query, Lazy Loading, Caching | `search_knowledge("N+1 query lazy loading caching performance")` |
| "Est-ce thread-safe ?", "Des problemes de concurrence ?" | Thread Safety, Race Condition | `search_knowledge("thread safety race condition shared mutable state")` |

### Revue et analyse de code

| Ce que vous dites (exemples) | Appel d'outil automatique |
|-------------------------------|--------------------------|
| Partager un extrait de code et demander une revue | `analyze_code(code)` → `suggest_refactorings(code)` |
| "Ameliorez ce code", "Refactorisez ça pour moi" | `suggest_refactorings(code)` |

### Discussions d'architecture

| Ce que vous dites (exemples) | Appel d'outil automatique |
|-------------------------------|--------------------------|
| "Microservices vs monolithe", "Comment devrions-nous separer ça ?" | `search_knowledge("monolith microservice decomposition bounded context")` |
| "Cette architecture est-elle correcte ?", "Revoyez ce design" | `search_knowledge("layered architecture coupling responsibility")` |

### Exemples de conversations reelles

**Exemple 1 — Detection d'un God Object :**

```
Vous : "UserService devient ingouvernable. Il gere l'authentification,
        la gestion des sessions, la verification d'email et les reinitialisations de mot de passe."

Claude (appelle automatiquement search_knowledge) :
  → Detecte : God Object / violation de Single Responsibility
  → Retourne : SMELL-03 (God Object), LAW-001 (SRP), RF-018 (Extract Class)

Claude : "UserService presente des caracteristiques de God Object (SMELL-03).
  Il viole le Principe de Responsabilite Unique (LAW-001).

  Refactoring recommandes :
  1. Extract Class (RF-018) — Creer SessionManager pour la logique de session
  2. Move Method (RF-023) — Deplacer la verification email vers EmailService
  3. Extract Interface (RF-042) — Definir l'interface AuthService

  Commencez par #2 pour des victoires rapides, puis #1 pour l'amelioration structurelle."
```

**Exemple 2 — Investigation de performance :**

```
Vous : "L'API de liste de produits est vraiment lente. Elle charge bien individuellement
        mais la page de liste prend 5+ secondes."

Claude (appelle automatiquement search_knowledge) :
  → Detecte : Probleme de requete N+1
  → Retourne : Motifs de performance, Lazy Loading, strategies de caching

Claude : "Cela ressemble a un probleme de requete N+1 — chaque produit declenche
  des appels DB individuels au lieu d'une requête par lot.

  Solutions depuis le graphe de connaissances :
  1. Batch loading / Eager loading — Recuperer les donnees liees en une seule requête
  2. Data Mapper (DP-019) — Decoupler la logique de requêtage des objets de domaine
  3. Appliquer Lazy Loading (RF-050) — Charger les relations a la demande uniquement

  Souhaitez-vous que je verifie le code pour les motifs N+1 ?"
```

**Exemple 3 — Revue de code avec extrait :**

```
Vous : [colle du code Python]
       "Pouvez-vous revoir ça ? Quelque chose ne va pas."

Claude (appelle automatiquement analyze_code → suggest_refactorings) :
  → analyze_code detecte : Long Method (confiance 0.87), Long Parameter List (0.92)
  → suggest_refactorings retourne des suggestions classees

Claude : "Trouve 2 code smells :

  1. Long Method (SMELL-01) — 45 lignes, complexite cyclomatique 12
     → Extract Method (RF-001), priorite 0.89

  2. Long Parameter List (SMELL-08) — 9 parametres
     → Introduce Parameter Object (RF-029), priorite 0.92

  Commencez par RF-029 (Parameter Object) — c'est la priorite la plus elevee
  et ça facilite le Extract Method subsequent."
```

---

## Prochaines etapes

1. **Essayer les agents** : Demandez a episteme-advisor "Devrais-je utiliser un Singleton ?"
2. **Analyser du code** : Collez une fonction et demandez a code-reviewer de verifier les smells
3. **Explorer le graphe** : Utilisez episteme-researcher pour trouver les relations entre motifs
4. **Flux personnalises** : Combiner les outils (analyze → suggest → search)

Pour plus d'exemples, voir :
- [Integration Alcove](./alcove-integration.md) — Connaissances d'equipe + Episteme
- [Configuration de la supervision](../monitoring/README.md) — Suivre l'utilisation des motifs
- [Reference API](./api.md) — Endpoints REST
