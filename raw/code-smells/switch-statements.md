# Switch Statements

## Overview
Complex switch/case or long if-else chains

## Symptoms
- Switch on type code
- Same switch duplicated across methods
- New cases require modification

## Why It Matters
OO Abuser smells indicate that object-oriented principles are not being applied correctly or are being actively circumvented. As an **OO Abuser**, this smell undermines the polymorphism and encapsulation that make OO designs resilient to change.

Specifically, it violates the **Open/Closed Principle (OCP)**, requiring modification of existing code to accommodate new behaviour.

## How to Fix
- **Replace Type Code With Class** (`RF-029`)

## Connections
**Violates:** Open/Closed Principle (`LAW-042-O`)

**Resolved by refactoring:** Replace Type Code With Class (`RF-029`)

---

*Based on: Refactoring (Fowler, 1999)*