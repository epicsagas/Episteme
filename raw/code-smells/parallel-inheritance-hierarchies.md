# Parallel Inheritance Hierarchies

## Overview
Every time you create a subclass for one class you find yourself needing to create a subclass for another

## Symptoms
- Two class hierarchies mirror each other
- Adding a subclass in one hierarchy requires adding one in another
- Class name prefixes are the same in both hierarchies

## Why It Matters
Change Preventer smells make the codebase brittle: a single logical change forces edits in many unrelated places. As a **Change Preventer**, this smell increases the risk and cost of every future modification, slowing down development over time.

Specifically, it violates the **Single Responsibility Principle (SRP)**, giving the construct more than one reason to change.

## How to Fix
- **Move Method** (`RF-016`)
- **Move Field** (`RF-015`)
- **Inline Class** (`RF-012`)

## Connections
**Violates:** Single Responsibility Principle (`LAW-042-S`)

**Resolved by refactoring:** Move Method (`RF-016`), Move Field (`RF-015`), Inline Class (`RF-012`)

---

*Based on: Refactoring (Fowler, 1999)*