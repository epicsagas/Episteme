# Consolidate Duplicate Conditional Fragments

## Motivation

Consolidate Duplicate Conditional Fragments removes identical code that appears in every branch of a conditional statement by hoisting it outside the branching structure. This duplication typically emerges through iterative feature work: a developer adds the same setup or teardown call to both the `if` and `else` paths and never circles back to factor it out. The result is a conditional where only the differentiating logic should occupy the branches, yet shared operations inflate each one. Readers must compare branch bodies line by line to confirm that the duplicated lines truly are identical, which wastes attention and increases the chance that a future edit updates one branch but not the other, introducing a divergence bug.

## Mechanics

1. Confirm that the duplicated fragment appears verbatim in every branch of the conditional, including any `else if` or `case` arms. Even a single-character difference disqualifies consolidation.
2. Determine whether the shared fragment sits at the beginning or the end of each branch. Fragments at the top of every branch move before the conditional; fragments at the bottom move after it.
3. When the duplicated code sits at the start of each branch, cut it from every branch and paste it immediately before the `if` or `switch` statement. Verify that no branch modifies variables the fragment depends on.
4. When the duplicated code sits at the end of each branch, move it to immediately after the conditional's closing brace. Ensure no branch contains an early return that would skip the relocated code.
5. If the duplicated fragment occupies a middle position within branches, assess whether reordering the branch logic allows placement at the top or bottom without changing semantics. When reordering is unsafe, apply Extract Method to the shared fragment, then call the new method from a single location outside the conditional.
6. Remove any branch that has become empty after extraction. If only one branch remains, replace the entire conditional with its sole body.
7. Run tests to verify that execution order is preserved, especially when the fragment interacts with variables that the conditional branches also modify.

## Indications

**Signs suggesting this refactoring:**
- Identical statements appearing at the top or bottom of every branch in an `if-else` or `switch` block
- A method call such as logging, sending, or persisting that appears unchanged in both the positive and negative paths
- Branch bodies that are mostly identical except for one or two differentiating lines buried among shared operations

**When to avoid:**
- The duplicated fragment depends on a variable whose value differs between branches at the point of duplication
- Moving the fragment would change the timing of a side effect that other code observes, such as a notification sent before a state mutation completes
- The conditional may short-circuit via early return, meaning a post-conditional placement would be skipped

## Trade-offs

Hoisting shared code outside a conditional reduces the total line count and eliminates the risk of branches drifting apart during future edits. The clarity gain is immediate: the conditional now expresses only what varies, and the surrounding code handles what is constant. However, the refactoring can obscure temporal ordering if a reader expects the shared fragment to execute within the branch context. When the shared code is a single trivial line, the reduction in duplication may not justify the cognitive shift of moving it away from its natural location. The technique also requires careful analysis when branches contain early returns, since moving a trailing fragment outside the conditional could silently alter which paths execute it. Apply the refactoring when the duplication spans multiple lines or when the shared operation clearly represents a cross-cutting concern like cleanup, logging, or result dispatch.

## Connections

This refactoring directly targets the Duplicate Code smell by collapsing repeated branch-internal logic into a single location. It pairs naturally with Extract Method, especially when the shared fragment is non-trivial and benefits from a named method of its own. Consolidate Conditional Expression is a related technique that merges entire branches sharing the same outcome, rather than shared code within differing branches. Replace Nested Conditional with Guard Clauses can simplify the conditional structure before or after consolidation, making it easier to identify and relocate shared fragments.

---

*Based on: Refactoring (Fowler, 1999)*
