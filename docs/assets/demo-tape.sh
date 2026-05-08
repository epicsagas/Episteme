#!/bin/bash
# Demo recording script for episteme
# Run: asciinema rec -c "USER=user HOST=localhost bash docs/assets/demo-tape.sh" docs/assets/demo.cast
set -e

cd "$(dirname "$0")/../.."

echo ""
sleep 1

# --- Part 1: Overview ---
echo "$ epis --version"
epis --version
sleep 2

echo ""
echo "$ epis stats"
epis stats
sleep 3

echo ""
sleep 1

# --- Part 2: Explore knowledge graph ---
echo "$ epis explore \"god object\""
epis explore "god object"
sleep 3

echo ""
echo "$ epis graph path LAW-001 RF-018"
epis graph path LAW-001 RF-018
sleep 3

sleep 2

clear

# --- Part 3: Analyze code ---
echo ""
echo "$ epis analyze src/domain/engine.rs"
epis analyze src/domain/engine.rs
sleep 3

echo ""
echo "$ epis infer src/domain/engine.rs --top-k 3"
epis infer src/domain/engine.rs --top-k 3
sleep 3

sleep 2

# --- Part 4: Launch Claude Code with episteme MCP ---
# Prompts to type after Claude Code opens (not shown on screen):
#   1. Find code smells in src/domain/engine.rs and suggest refactorings
#   2. What refactorings solve the God Object smell? Show related laws.
#   3. Find the path between SRP and Extract Class in the knowledge graph
echo ""
echo "$ claudy zai --yolo"
claudy zai --yolo

sleep 2

clear

echo ""
echo "$ epis stats"
epis stats

echo ""
echo "$ epis explore \"extract method\""
epis explore "extract method"

sleep 2
