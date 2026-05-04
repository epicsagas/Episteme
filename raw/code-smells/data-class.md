# Data Class

## Overview
Class contains only fields with getters and setters but no meaningful behavior

## Symptoms
- High field-to-method ratio
- Methods are mostly getters and setters
- No behavior or business logic in the class

## Why It Matters
Dispensable smells point to code that is unnecessary and whose removal would make the codebase cleaner and easier to understand. As a **Dispensable**, this smell adds noise, cognitive overhead, and maintenance burden without providing corresponding value.

Specifically, it violates the **DRY principle** (Don't Repeat Yourself), promoting unnecessary duplication and divergence.

## How to Fix
- **Replace Data Value With Object** (`RF-026`)
- **Change Reference To Value** (`RF-019`)

## Connections
**Violates:** Dry (`LAW-040`)

**Resolved by refactoring:** Replace Data Value With Object (`RF-026`), Change Reference To Value (`RF-019`)

---

*Based on: Refactoring (Fowler, 1999)*