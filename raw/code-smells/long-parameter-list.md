# Long Parameter List

## Overview
More than 3-4 parameters in a method

## Symptoms
- Method has more than 3-4 parameters
- Parameter list keeps growing
- Parameters form natural groups

## Why It Matters
Bloater smells arise when a method, class, or parameter list grows to a size that makes it hard to read and maintain. As a **Bloater**, this smell degrades comprehensibility and increases the cognitive load required to understand even small changes.

Specifically, it violates the **KISS principle** (Keep It Simple), adding unnecessary complexity.

## How to Fix
- **Replace Conditional With Polymorphism** (`RF-039`)
- **Introduce Null Object** (`RF-037`)

## Connections
**Violates:** Kiss (`LAW-041`)

**Resolved by refactoring:** Replace Conditional With Polymorphism (`RF-039`), Introduce Null Object (`RF-037`)

---

*Based on: Refactoring (Fowler, 1999)*