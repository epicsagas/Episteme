# Data Clumps

## Overview
Different parts of code contain identical groups of variables

## Symptoms
- Same group of parameters in multiple methods
- Same fields in multiple classes
- Grouped data used together

## Why It Matters
Bloater smells arise when a method, class, or parameter list grows to a size that makes it hard to read and maintain. As a **Bloater**, this smell degrades comprehensibility and increases the cognitive load required to understand even small changes.

Specifically, it violates the **DRY principle** (Don't Repeat Yourself), promoting unnecessary duplication and divergence.

## How to Fix
- **Introduce Parameter Object** (`RF-043`)
- **Preserve Whole Object** (`RF-045`)
- **Extract Class** (`RF-010`)
- **Move Method** (`RF-016`)

## Connections
**Violates:** Dry (`LAW-040`)

**Resolved by refactoring:** Introduce Parameter Object (`RF-043`), Preserve Whole Object (`RF-045`), Extract Class (`RF-010`), Move Method (`RF-016`)

---

*Based on: Refactoring (Fowler, 1999)*