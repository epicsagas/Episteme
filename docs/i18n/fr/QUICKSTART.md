# Episteme — Guide de demarrage rapide

Devenez operationnel avec Episteme en moins de 2 minutes.

---

## Prerequis

- **Rust 1.95+** (edition 2024 requise) — [Installer via rustup](https://rustup.rs)
- Connexion Internet (pour le telechargement initial des donnees)

---

## Option 1 : Integration a un outil IA (Recommande)

**Ideal pour :** Utilisateurs de Claude Code, Cursor, Codex, Gemini

```bash
# 1. Installer Episteme
cargo install --git https://github.com/epicsagas/Episteme

# 2. Installer dans votre outil IA (telecharge les donnees, configure MCP, copie les agents)
epis install claude      # Claude Code
epis install cursor      # Cursor
epis install codex       # OpenAI Codex
epis install gemini      # Antigravity
epis install all         # Tous les outils a la fois
```

> Si `epis install claude` echoue a telecharger les donnees, utilisez l'installation depuis les sources ci-dessous.

**C'est tout.** Redemarrez votre outil IA et Episteme est actif.

---

## Option 2 : Docker (Rust non requis)

```bash
docker-compose up -d

# Acces
# API:       http://localhost:8000
# Sante:     http://localhost:8000/health
```

Pour l'integration MCP via Docker, ajoutez a votre configuration MCP :
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

---

## Option 3 : Depuis les sources

```bash
git clone https://github.com/epicsagas/Episteme.git
cd Episteme

# Compilation
cargo build --release

# Peupler les donnees et construire la base vectorielle (la compilation se lance automatiquement)
./target/release/epis install --local
```

---

## Visualisation du graphe

Episteme inclut une visionneuse interactive basee sur D3-force :

```bash
episteme web               # par defaut : http://localhost:8080
episteme web --port 9001   # port personnalise
episteme web --host 0.0.0.0 --port 8080  # exposer sur le reseau
```

---

## Commandes courantes

```bash
# Analyser le code pour detecter les code smells
epis analyze my_code.py --language python
epis analyze my_code.py --json

# Obtenir des suggestions de refactoring
episteme infer my_code.py --top-k 5

# Explorer le graphe de connaissances
epis explore "strategy pattern"
epis graph path DP-005 RF-001

# Demarrer les serveurs
epis api              # API REST sur :8000
episteme mcp --http       # Serveur MCP sur :43175
episteme web --port 8080  # Interface Web

# Daemon MCP en arriere-plan (proxy HTTP)
epis service start
epis service status
epis service stop

# Creer une archive de release
episteme dist --out-dir release
```

---

## Depannage

### « Base de donnees introuvable »
```bash
epis install claude   # re-telecharger l'archive de donnees
# ou
epis install --local
```

### « Port deja utilise »
```bash
episteme web --port 9001
epis api --port 9000
```

---

## Prochaines etapes

- **[README](../../README.md)** — Presentation complete des fonctionnalites et de l'architecture
- **[Guide d'integration MCP](./mcp-integration-guide.md)** — Reference des outils et exemples d'agents
- **[Reference API](./api.md)** — Endpoints REST
- **[Contribuer](../../CONTRIBUTING.md)** — Flux de developpement
