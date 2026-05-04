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
- **Self Encapsulate Field** (`RF-032`)
- **Encapsulate Field** (`RF-024`)

## Connections
**Violates:** Single Responsibility Principle (`LAW-042-S`)

**Resolved by refactoring:** Self Encapsulate Field (`RF-032`), Encapsulate Field (`RF-024`)

---

*Based on: Refactoring (Fowler, 1999)*