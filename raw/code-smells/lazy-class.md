# Lazy Class

## Overview
Class that does too little and does not justify its existence

## Symptoms
- Very few methods or lines of code
- Class is nearly empty or trivial
- Could be merged into another class

## Why It Matters
Dispensable smells point to code that is unnecessary and whose removal would make the codebase cleaner and easier to understand. As a **Dispensable**, this smell adds noise, cognitive overhead, and maintenance burden without providing corresponding value.

Specifically, it violates the **DRY principle** (Don't Repeat Yourself), promoting unnecessary duplication and divergence.

## How to Fix
- **Inline Class** (`RF-012`)
- **Collapse Hierarchy** (`RF-055`)

## Connections
**Violates:** Dry (`LAW-040`)

**Resolved by refactoring:** Inline Class (`RF-012`), Collapse Hierarchy (`RF-055`)

---

*Based on: Refactoring (Fowler, 1999)*