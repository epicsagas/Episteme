# Feature Envy

## Overview
Method accesses data of another object more than its own

## Symptoms
- Method uses many getters from another class
- Method is more interested in other class than its own
- Method manipulates foreign data

## Why It Matters
Coupler smells indicate excessive coupling between classes or modules, reducing cohesion and making it difficult to change one part of the system without affecting others. As a **Coupler**, this smell works against the goal of designing components that can be understood, tested, and modified in isolation.

Specifically, it violates the **Law of Demeter**, exposing implementation details across object boundaries.

## How to Fix
- **Extract Method** (`RF-001`)
- **Change Reference To Value** (`RF-019`)

## Connections
**Violates:** Law Of Demeter (`LAW-043`)

**Resolved by refactoring:** Extract Method (`RF-001`), Change Reference To Value (`RF-019`)

---

*Based on: Refactoring (Fowler, 1999)*