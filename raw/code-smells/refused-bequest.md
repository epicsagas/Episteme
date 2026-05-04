# Refused Bequest

## Overview
Subclass inherits methods or data it does not need or want from its parent

## Symptoms
- Subclass inherits methods or data it does not use
- Subclass overrides inherited behaviour to do nothing or throw
- Base class carries baggage that only some subclasses need

## Why It Matters
Inheritance Abuser smells signal that the inheritance hierarchy is being misused or that subclasses do not honour the contract defined by their superclass. As an **Inheritance Abuser**, this smell can lead to fragile hierarchies and subtle runtime bugs when callers rely on substitutability that the subclass does not provide.

Specifically, it violates the **Liskov Substitution Principle (LSP)**, breaking substitutability between a type and its subtypes.

## How to Fix
- **Self Encapsulate Field** (`RF-032`)

## Connections
**Violates:** Liskov Substitution Principle (`LAW-042-L`)

**Resolved by refactoring:** Self Encapsulate Field (`RF-032`)

---

*Based on: Refactoring (Fowler, 1999)*