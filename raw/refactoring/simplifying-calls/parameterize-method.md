# Parameterize Method

## Motivation

Parameterize Method consolidates several near-identical methods into a single method whose varying values are supplied through a parameter. The canonical signal is a cluster of methods whose bodies differ only by a magic number, a string literal, or a fixed expression: `baseSalary()`, `bonusSalary()`, and `totalSalary()` might all execute the same calculation with different multipliers. Each variant adds surface area for bugs because a fix applied to one must be manually replicated to the others, and a missed replication silently diverges behavior.

By introducing a parameter that captures the varying dimension, the duplicated logic collapses into one authoritative location. New variants no longer require a new method; they emerge from a different argument value. This reduction directly attacks the Duplicate Code smell and makes the method's intent explicit: the parameter name documents what dimension varies across the former variants.

## Mechanics

1. Choose one of the similar methods as the canonical implementation and apply Extract Method to isolate its logic into a new parameterized method placed where all variants can reach it.
2. Replace every hardcoded value that distinguishes the variants with a parameter reference. Assign a meaningful name to the parameter so that call sites read naturally.
3. Redirect each original method's callers to the new parameterized version, passing the appropriate argument for each case.
4. Delete the now-empty original methods. Run the full test suite to verify that each former caller produces identical results through the unified method.

## Indications

**Signs suggesting this refactoring:**
- Two or more methods share structure and differ only in embedded constants, coefficients, or string literals.
- Adding a new variant forces creation of yet another copy-paste method.
- Review comments or naming conventions reveal a family of methods following the same pattern, such as `computeWeekly`, `computeMonthly`, `computeYearly`.

**When to avoid:**
- The methods are similar now but are expected to diverge in logic as the domain evolves; premature consolidation would couple unrelated futures.
- The resulting parameterized method would require a complex conditional or switch on the parameter value, indicating that Replace Parameter with Explicit Methods is the better direction.
- The variation is behavioral rather than numeric, and the parameter would act as a mode flag that obscures what the method actually does for a given call.

## Trade-offs

Consolidation yields a single source of truth for the shared algorithm, which dramatically reduces the risk of inconsistent fixes and simplifies future extensions. Callers gain flexibility because they can pass arbitrary values rather than being locked to a predefined set of methods. The downside surfaces when the parameter acts as a selector for fundamentally different behaviors rather than different data: a method that switches on its argument to choose among unrelated code paths becomes harder to understand than a set of clearly named methods. In that scenario, the refactoring exchanges duplication for a different form of complexity. The technique also introduces a minor cognitive cost at each call site, where the reader must infer what a numeric or string argument means without the explicitness of a dedicated method name.

## Connections

Parameterize Method is the inverse of Replace Parameter with Explicit Methods. It frequently follows Extract Method as a second step, once the common structure of several routines becomes visible. The technique eliminates Duplicate Code and can be a precursor to Form Template Method, where the parameterized algorithm is pulled into a superclass and individual subclasses override specific steps. When the varying dimension is an object rather than a primitive, the technique edges toward Strategy Pattern territory, where behavior is injected rather than selected by a scalar parameter. Introduce Parameter Object may further simplify the result when multiple parameters accumulate during consolidation.

---

*Based on: Refactoring (Fowler, 1999)*
