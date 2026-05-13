# Divergent Change

## Overview
One class commonly changed in different ways for different reasons

## Symptoms
- Class changes for multiple reasons
- Different methods change for different features
- Multiple responsibilities

## Why It Matters
Change Preventer smells make the codebase brittle: a single logical change forces edits in many unrelated places. As a **Change Preventer**, this smell increases the risk and cost of every future modification, slowing down development over time.

Specifically, it violates the **Single Responsibility Principle (SRP)**, giving the construct more than one reason to change.

## How to Fix
- **Extract Class** (`RF-010`)
- **Move Field** (`RF-015`)
- **Move Method** (`RF-016`)

## Connections
**Violates:** Single Responsibility Principle (`LAW-042-S`)

**Resolved by refactoring:** Extract Class (`RF-010`), Move Field (`RF-015`), Move Method (`RF-016`)

---

*Based on: Refactoring (Fowler, 1999)*