# Primitive Obsession

## Overview
Use of primitives instead of small objects for simple tasks

## Symptoms
- Using primitives for domain concepts
- Simulation of types via constants
- Type code as field names

## Why It Matters
Bloater smells arise when a method, class, or parameter list grows to a size that makes it hard to read and maintain. As a **Bloater**, this smell degrades comprehensibility and increases the cognitive load required to understand even small changes.

Specifically, it violates the **DRY principle** (Don't Repeat Yourself), promoting unnecessary duplication and divergence.

## How to Fix
- **Replace Data Value With Object** (`RF-026`)

## Connections
**Violates:** Dry (`LAW-040`)

**Resolved by refactoring:** Replace Data Value With Object (`RF-026`)

---

*Based on: Refactoring (Fowler, 1999)*