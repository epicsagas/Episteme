# Long Method

## Overview
Method contains too many lines of code (typically >20-30 lines)

## Symptoms
- Method exceeds 20-30 lines
- Multiple levels of nesting
- Requires inline comments to explain sections

## Why It Matters
Bloater smells arise when a method, class, or parameter list grows to a size that makes it hard to read and maintain. As a **Bloater**, this smell degrades comprehensibility and increases the cognitive load required to understand even small changes.

Specifically, it violates the **Single Responsibility Principle (SRP)**, giving the construct more than one reason to change, and the **KISS principle** (Keep It Simple), adding unnecessary complexity.

## How to Fix
- **Extract Method** (`RF-001`)
- **Split Temporary Variable** (`RF-008`)

## Connections
**Violates:** Single Responsibility Principle (`LAW-042-S`), Kiss (`LAW-041`)

**Resolved by refactoring:** Extract Method (`RF-001`), Split Temporary Variable (`RF-008`)

---

*Based on: Refactoring (Fowler, 1999)*