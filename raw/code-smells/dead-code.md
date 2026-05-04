# Dead Code

## Overview
Code that is no longer used or reachable and could be safely removed

## Symptoms
- Unreachable code after a return or throw
- Variables, methods, or classes that are never referenced
- Commented-out code left in the codebase

## Why It Matters
Dispensable smells point to code that is unnecessary and whose removal would make the codebase cleaner and easier to understand. As a **Dispensable**, this smell adds noise, cognitive overhead, and maintenance burden without providing corresponding value.

Specifically, it violates the **DRY principle** (Don't Repeat Yourself), promoting unnecessary duplication and divergence.

## How to Fix
- **Consolidate Duplicate Conditional Fragments** (`RF-034`)

## Connections
**Violates:** Dry (`LAW-040`)

**Resolved by refactoring:** Consolidate Duplicate Conditional Fragments (`RF-034`)

---

*Based on: Refactoring (Fowler, 1999)*