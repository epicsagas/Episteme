# Inappropriate Intimacy

## Overview
Two classes access each other's private parts more than necessary, creating excessive coupling

## Symptoms
- Classes access each other's private fields or methods
- Bidirectional dependencies between classes
- Excessive coupling making changes in one class affect the other

## Why It Matters
Coupler smells indicate excessive coupling between classes or modules, reducing cohesion and making it difficult to change one part of the system without affecting others. As a **Coupler**, this smell works against the goal of designing components that can be understood, tested, and modified in isolation.

Specifically, it violates the **Law of Demeter**, exposing implementation details across object boundaries.

## How to Fix
- **Change Unidirectional Association To Bidirectional** (`RF-020`)
- **Self Encapsulate Field** (`RF-032`)

## Connections
**Violates:** Law Of Demeter (`LAW-043`)

**Resolved by refactoring:** Change Unidirectional Association To Bidirectional (`RF-020`), Self Encapsulate Field (`RF-032`)

---

*Based on: Refactoring (Fowler, 1999)*