# Speculative Generality

## Overview
Unused abstraction created for anticipated future needs that never materialise

## Symptoms
- Abstract class or interface with only one implementation
- Class is defined but never or rarely used
- Over-engineered hooks for hypothetical extensions

## Why It Matters
Dispensable smells point to code that is unnecessary and whose removal would make the codebase cleaner and easier to understand. As a **Dispensable**, this smell adds noise, cognitive overhead, and maintenance burden without providing corresponding value.

Specifically, it violates the **DRY principle** (Don't Repeat Yourself), promoting unnecessary duplication and divergence.

## How to Fix
- **Consolidate Duplicate Conditional Fragments** (`RF-034`)
- **Encapsulate Field** (`RF-024`)

## Connections
**Violates:** Dry (`LAW-040`)

**Resolved by refactoring:** Consolidate Duplicate Conditional Fragments (`RF-034`), Encapsulate Field (`RF-024`)

---

*Based on: Refactoring (Fowler, 1999)*