# Comments

## Overview
Comment used as deodorant to explain confusing code rather than to convey intent that code cannot express

## Symptoms
- Long comment block explaining what the code does
- Comment that would be unnecessary if the code were clearer
- TODO or FIXME comments persisting without resolution

## Why It Matters
Dispensable smells point to code that is unnecessary and whose removal would make the codebase cleaner and easier to understand. As a **Dispensable**, this smell adds noise, cognitive overhead, and maintenance burden without providing corresponding value.

Specifically, it violates the **DRY principle** (Don't Repeat Yourself), promoting unnecessary duplication and divergence.

## How to Fix
- **Extract Method** (`RF-001`)
- **Decompose Conditional** (`RF-035`)

## Connections
**Violates:** Dry (`LAW-040`)

**Resolved by refactoring:** Extract Method (`RF-001`), Decompose Conditional (`RF-035`)

---

*Based on: Refactoring (Fowler, 1999)*