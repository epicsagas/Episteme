# Shotgun Surgery

## Overview
Single change requires many small changes to many different classes

## Symptoms
- One change affects many classes
- Related functionality scattered
- Hard to find all places to change

## Why It Matters
Change Preventer smells make the codebase brittle: a single logical change forces edits in many unrelated places. As a **Change Preventer**, this smell increases the risk and cost of every future modification, slowing down development over time.

Specifically, it violates the **Single Responsibility Principle (SRP)**, giving the construct more than one reason to change.

## How to Fix
- **Extract Class** (`RF-010`)
- **Extract Method** (`RF-001`)
- **Move Method** (`RF-016`)
- **Move Field** (`RF-015`)

## Connections
**Violates:** Single Responsibility Principle (`LAW-042-S`)

**Resolved by refactoring:** Extract Class (`RF-010`), Extract Method (`RF-001`), Move Method (`RF-016`), Move Field (`RF-015`)

---

*Based on: Refactoring (Fowler, 1999)*