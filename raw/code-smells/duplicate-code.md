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
- **Change Reference To Value** (`RF-019`)
- **Remove Parameter** (`RF-046`)

## Connections
**Violates:** Dry (`LAW-040`)

**Resolved by refactoring:** Extract Method (`RF-001`), Change Reference To Value (`RF-019`), Remove Parameter (`RF-046`)

---

*Based on: Refactoring (Fowler, 1999)*