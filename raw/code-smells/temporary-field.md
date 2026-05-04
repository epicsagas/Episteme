# Temporary Field

## Overview
Object field set only in certain circumstances, leaving it empty or null the rest of the time

## Symptoms
- Field is only set in specific code paths
- Field is null or empty under normal conditions
- Code checks whether field is set before using it

## Why It Matters
OO Abuser smells indicate that object-oriented principles are not being applied correctly or are being actively circumvented. As an **OO Abuser**, this smell undermines the polymorphism and encapsulation that make OO designs resilient to change.

Specifically, it violates the **DRY principle** (Don't Repeat Yourself), promoting unnecessary duplication and divergence.

## How to Fix
- **Self Encapsulate Field** (`RF-032`)

## Connections
**Violates:** Dry (`LAW-040`)

**Resolved by refactoring:** Self Encapsulate Field (`RF-032`)

---

*Based on: Refactoring (Fowler, 1999)*