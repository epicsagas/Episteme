# Large Class

## Overview
Class contains too many fields, methods, or lines of code

## Symptoms
- Class exceeds 200-300 lines
- Multiple unrelated responsibilities
- High instance variable count

## Why It Matters
Bloater smells arise when a method, class, or parameter list grows to a size that makes it hard to read and maintain. As a **Bloater**, this smell degrades comprehensibility and increases the cognitive load required to understand even small changes.

Specifically, it violates the **Single Responsibility Principle (SRP)**, giving the construct more than one reason to change.

## How to Fix
- **Extract Class** (`RF-010`)
- **Extract Method** (`RF-001`)
- **Move Method** (`RF-016`)
- **Extract Subclass** (`RF-057`)
- **Extract Superclass** (`RF-058`)
- **Replace Conditional With Polymorphism** (`RF-039`)
- **Push Down Field** (`RF-063`)
- **Push Down Method** (`RF-064`)

## Connections
**Violates:** Single Responsibility Principle (`LAW-042-S`)

**Resolved by refactoring:** Extract Class (`RF-010`), Extract Method (`RF-001`), Move Method (`RF-016`), Extract Subclass (`RF-057`), Extract Superclass (`RF-058`), Replace Conditional With Polymorphism (`RF-039`), Push Down Field (`RF-063`), Push Down Method (`RF-064`)

---

*Based on: Refactoring (Fowler, 1999)*