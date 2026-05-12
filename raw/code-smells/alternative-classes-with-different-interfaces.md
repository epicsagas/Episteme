# Alternative Classes with Different Interfaces

## Overview
Two classes perform identical or similar functions but have different method signatures

## Symptoms
- Two classes do similar things but have different method names
- Classes could be used interchangeably but lack a common interface
- Clients must know which class to use and adapt to each

## Why It Matters
OO Abuser smells indicate that object-oriented principles are not being applied correctly or are being actively circumvented. As an **OO Abuser**, this smell undermines the polymorphism and encapsulation that make OO designs resilient to change.

Specifically, it violates the **Interface Segregation Principle (ISP)**, forcing clients to depend on interfaces they do not use.

## How to Fix
- **Rename Method** (`RF-048`)
- **Extract Interface** (`RF-056`)
- **Extract Method** (`RF-001`)

## Connections
**Violates:** Interface Segregation Principle (`LAW-042-I`)

**Resolved by refactoring:** Rename Method (`RF-048`), Extract Interface (`RF-056`), Extract Method (`RF-001`)

---

*Based on: Refactoring (Fowler, 1999)*