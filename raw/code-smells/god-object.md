# God Object

## Overview
Class that knows and does too much, centralising an excessive number of responsibilities

## Symptoms
- Excessive method count (>30)
- Excessive field count (>20)
- Very high lines of code (>500)
- Extreme cyclomatic complexity

## Why It Matters
Bloater smells arise when a method, class, or parameter list grows to a size that makes it hard to read and maintain. As a **Bloater**, this smell degrades comprehensibility and increases the cognitive load required to understand even small changes.

Specifically, it violates the **Single Responsibility Principle (SRP)**, giving the construct more than one reason to change, and the **Open/Closed Principle (OCP)**, requiring modification of existing code to accommodate new behaviour.

## How to Fix
- **Self Encapsulate Field** (`RF-032`)
- **Encapsulate Field** (`RF-024`)

## Connections
**Violates:** Single Responsibility Principle (`LAW-042-S`), Open/Closed Principle (`LAW-042-O`)

**Resolved by refactoring:** Self Encapsulate Field (`RF-032`), Encapsulate Field (`RF-024`)

---

*Based on: Refactoring (Fowler, 1999)*