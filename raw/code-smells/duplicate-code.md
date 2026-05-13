# Duplicate Code

## Overview
Same code structure appears in multiple places

## Symptoms
- Identical expressions in multiple methods
- Similar code in sibling subclasses
- Copied and pasted code

## Why It Matters
Dispensable smells point to code that is unnecessary and whose removal would make the codebase cleaner and easier to understand. As a **Dispensable**, this smell adds noise, cognitive overhead, and maintenance burden without providing corresponding value.

Specifically, it violates the **DRY principle** (Don't Repeat Yourself), promoting unnecessary duplication and divergence.

## How to Fix
- **Extract Method** (`RF-001`)
- **Form Template Method** (`RF-059`)
- **Extract Superclass** (`RF-058`)
- **Pull Up Method** (`RF-062`)
- **Pull Up Field** (`RF-061`)
- **Parameterize Method** (`RF-044`)
- **Replace Magic Number With Symbolic Constant** (`RF-027`)

## Connections
**Violates:** Dry (`LAW-040`)

**Resolved by refactoring:** Extract Method (`RF-001`), Form Template Method (`RF-059`), Extract Superclass (`RF-058`), Pull Up Method (`RF-062`), Pull Up Field (`RF-061`), Parameterize Method (`RF-044`), Replace Magic Number With Symbolic Constant (`RF-027`)

---

*Based on: Refactoring (Fowler, 1999)*