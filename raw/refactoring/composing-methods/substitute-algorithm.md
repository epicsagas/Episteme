# Substitute Algorithm

## Motivation
Substitute Algorithm replaces the entire implementation of a method with a fundamentally different approach that produces the same results. The need arises when the original algorithm was written for clarity during prototyping but is now a performance bottleneck, when new library or framework APIs provide a canonical solution that renders hand-written logic obsolete, or when changing requirements have stretched the original algorithm beyond its design limits and patching it incrementally would produce more complexity than starting fresh. Rather than surgically modifying a method that is already difficult to understand, the developer rewrites it from scratch with a cleaner strategy and validates that the new version produces identical outputs.

This refactoring differs from incremental improvements like Extract Method or Replace Temp with Query because it replaces the entire computational core rather than restructuring the existing code. It is the correct choice when the problem is not how the code is organized but what the code does at an algorithmic level — switching from linear search to hash lookup, replacing a handwritten sort with a library sort, or exchanging a brute-force calculation for a closed-form formula.

## Mechanics
1. Before touching the implementation, ensure comprehensive tests exist that capture the method's expected behavior across its full input domain, including edge cases and boundary values.
2. Simplify the existing method as much as possible using supporting refactorings — Extract Method for peripheral logic, Replace Temp with Query for clarity — so the core algorithm is isolated and easy to compare.
3. Write the new algorithm in place of the old one, aiming for a clean implementation unconstrained by the structure of the original.
4. Run the test suite against the new implementation. If tests fail, investigate whether the new algorithm has a defect or whether the tests themselves encoded assumptions about the old algorithm's internals rather than its contract.
5. Compare the results of both algorithms on a representative sample of inputs — logging or diffing outputs side by side — to catch behavioral drift that unit tests may not cover.
6. Once the new algorithm passes all tests and produces equivalent results, remove the old implementation entirely and clean up any temporary scaffolding introduced during the transition.

## Indications

**Signs suggesting this refactoring:**
- A method's performance profile is dominated by an algorithm whose time complexity no longer meets the application's needs — for example, O(n^2) processing on a dataset that has grown by orders of magnitude
- A standard library or framework now provides a battle-tested implementation of the same computation, eliminating the need for custom code
- The existing algorithm has accumulated so many patches and special cases that its control flow resembles a state machine more than a coherent procedure
- Changing requirements — such as supporting a new data format or a different ordering rule — cannot be grafted onto the current approach without a complete redesign

**When to avoid:**
- The existing algorithm is correct and readable, and the motivation is purely aesthetic — rewriting working code for stylistic preference introduces risk without proportional benefit
- The method is in a critical hot path and the new algorithm's performance characteristics are not yet benchmarked under realistic load
- The method is part of a published API where subtle behavioral differences — even ones that don't break tests — could affect downstream consumers

## Trade-offs
The primary benefit is a leap in code quality that incremental refactorings cannot achieve: a well-chosen replacement algorithm can reduce line count by half, improve time complexity by an order of magnitude, or eliminate an entire category of bugs. Library-based replacements also reduce maintenance burden by delegating ongoing correctness to the library maintainers. The risk is behavioral regression: even with thorough tests, subtle differences in floating-point handling, null treatment, or edge-case ordering can slip through. The mitigation is a systematic comparison phase where both algorithms run in parallel on real data before the old one is removed. This refactoring also demands discipline around scope — the developer must resist the temptation to change the method's contract or add new features during the replacement, which should be separate commits with separate tests.

## Connections
Substitute Algorithm often follows a series of preparatory refactorings — Extract Method, Replace Temp with Query, and Split Temporary Variable — that isolate the algorithmic core from peripheral logic, making the boundary between "what to replace" and "what to keep" explicit. It directly addresses the Long Method and Duplicate Code smells when the duplication is algorithmic rather than structural. The technique aligns with the Strategy pattern: when multiple algorithms exist for the same computation, each can be encapsulated in its own strategy object, and Substitute Algorithm becomes the act of swapping one strategy for another. When the new algorithm comes from a library, the refactoring overlaps with Introduce Foreign Method or Replace Constructor with Factory Method depending on how the library is integrated.

---

*Based on: Refactoring (Fowler, 1999)*
