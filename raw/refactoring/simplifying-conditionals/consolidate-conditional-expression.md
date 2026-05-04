# Consolidate Conditional Expression

## Motivation

Consolidate Conditional Expression merges a series of separate conditional checks that all produce the same outcome into one unified boolean expression. This situation arises when a method contains multiple consecutive `if` statements that each trigger an identical action, such as returning the same value or throwing the same exception. The duplicated result obscures the programmer's real intent: a single eligibility rule composed of several criteria. Scattered checks force readers to mentally accumulate conditions rather than grasping the rule at a glance. The pattern commonly appears in validation layers, eligibility calculations, and guard logic where business rules involve multiple disqualifying factors evaluated in sequence.

## Mechanics

1. Inspect every conditional branch to confirm they share the same body and contain no side effects. If a branch modifies state, isolate that logic first using Extract Method.
2. Identify whether the checks are sequential early returns joined by logical OR, or nested guards joined by logical AND. Consecutive `if (x) return 0; if (y) return 0;` collapses with OR; nested `if (x) { if (y) { ... } }` collapses with AND.
3. Combine the expressions into a single `if` statement using the appropriate logical operator, keeping parentheses explicit to preserve evaluation semantics.
4. Extract the combined boolean expression into a well-named query method, such as `isNotEligibleForDisability()`, using Extract Method. The method name should describe the business rule rather than the implementation detail.
5. Verify that short-circuit evaluation semantics remain correct: if the original code relied on left-to-right evaluation order, ensure the consolidated expression preserves that dependency.
6. Run the test suite to confirm behavioral equivalence, paying special attention to edge cases where one condition's truth value depends on a prior check having passed.

## Indications

**Signs suggesting this refactoring:**
- Multiple consecutive conditional statements that return the same value or perform the same action
- Nested `if` blocks where every path converges on identical logic
- A sequence of guard conditions whose individual purpose is hard to distinguish from the shared outcome

**When to avoid:**
- The conditional bodies differ in subtle ways, even if they appear similar at first glance
- The individual checks serve distinct logging or auditing purposes that must remain separate
- Combining the expressions would produce a single line so long that readability degrades significantly

## Trade-offs

Consolidation trades multiple small, self-evident checks for one compact expression whose meaning is captured entirely by the extracted method name. The gain is proportional to how well that name communicates intent: a vague name like `checkConditions()` negates the benefit entirely. Combining conditions also risks introducing subtle bugs when short-circuit evaluation hides side effects or when one condition depends on a variable initialized by a prior branch. In codebases where individual conditions change at different rates, a single combined expression may actually increase coupling, since any modification requires understanding the full boolean formula rather than editing one isolated check. Apply this refactoring when the conditions are stable and form a coherent business rule.

## Connections

This technique directly addresses the Duplicate Code smell by eliminating repeated conditional bodies. It relies on Extract Method to give the consolidated expression a meaningful name. Consolidate Duplicate Conditional Fragments is a sibling refactoring that tackles duplicated code inside branches rather than duplicated branches themselves. Replace Nested Conditional with Guard Clauses often precedes this refactoring by flattening nested structures into sequential checks that can then be consolidated. Decompose Conditional moves in the opposite direction, splitting a complex condition into named pieces for clarity.

---

*Based on: Refactoring (Fowler, 1999)*
