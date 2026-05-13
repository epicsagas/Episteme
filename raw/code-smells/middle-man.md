# Middle Man

## Overview
Class delegates most of its work to another class, adding no value of its own

## Symptoms
- Most methods are single-line delegations
- High delegation-to-method ratio
- Class can be removed without losing behaviour

## Why It Matters
Coupler smells indicate excessive coupling between classes or modules, reducing cohesion and making it difficult to change one part of the system without affecting others. As a **Coupler**, this smell works against the goal of designing components that can be understood, tested, and modified in isolation.

Specifically, it violates the **Law of Demeter**, exposing implementation details across object boundaries.

## How to Fix
- **Remove Middle Man** (`RF-017`)
- **Replace Delegation With Inheritance** (`RF-065`)

## Connections
**Violates:** Law Of Demeter (`LAW-043`)

**Resolved by refactoring:** Remove Middle Man (`RF-017`), Replace Delegation With Inheritance (`RF-065`)

---

*Based on: Refactoring (Fowler, 1999)*