# Message Chains

## Overview
Client asks object for another object, which asks for another, etc.

## Symptoms
- Long chains like a.getB().getC().getD()
- Client depends on navigation structure
- Changes ripple through chain

## Why It Matters
Coupler smells indicate excessive coupling between classes or modules, reducing cohesion and making it difficult to change one part of the system without affecting others. As a **Coupler**, this smell works against the goal of designing components that can be understood, tested, and modified in isolation.

Specifically, it violates the **Law of Demeter**, exposing implementation details across object boundaries.

## How to Fix
- **Hide Delegate** (`RF-011`)
- **Move Method** (`RF-016`)
- **Encapsulate Collection** (`RF-023`)

## Connections
**Violates:** Law Of Demeter (`LAW-043`)

**Resolved by refactoring:** Hide Delegate (`RF-011`), Move Method (`RF-016`), Encapsulate Collection (`RF-023`)

---

*Based on: Refactoring (Fowler, 1999)*