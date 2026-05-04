# Inline Temp

## Motivation
Inline Temp eliminates a temporary variable that adds indirection without contributing clarity. These variables typically hold the result of a single expression and are used only once, often on the very next line. When the variable name merely restates what the expression already communicates — such as assigning `totalPrice = order.totalPrice()` — the variable is pure noise that forces the reader to track an extra binding. Removing it tightens the code and brings the expression directly into the context where it is consumed.

This refactoring is most often used as a preparatory step within a larger refactoring sequence. Before applying Replace Temp with Query, the developer first inlines the temporary variable to see every place the expression is evaluated, then extracts a query method from those sites. Similarly, before performing Extract Method on a block that reads a temporary variable, inlining the variable removes a local-state dependency that would otherwise complicate the extraction's parameter list.

## Mechanics
1. Locate the temporary variable declaration and confirm it is assigned exactly once within the method — if it receives multiple assignments, apply Split Temporary Variable first.
2. Identify every reference to the variable within the method and verify that replacing each with the original expression will not change evaluation semantics, paying special attention to side effects and short-circuit evaluation.
3. Replace each reference with a copy of the expression from the right-hand side of the assignment, preserving any necessary parentheses to maintain evaluation order.
4. Remove the variable declaration and its assignment statement, then run tests to confirm the refactoring preserved behavior.

## Indications

**Signs suggesting this refactoring:**
- A temporary variable is assigned once and referenced once, with no other code between the assignment and its use
- The variable name restates the expression in natural language without adding domain insight — for example, `isLarge = size > MAX_THRESHOLD`
- The variable is a remnant of an earlier refactoring that left behind an unnecessary intermediate binding

**When to avoid:**
- The expression is expensive to compute and is referenced multiple times, in which case the temporary variable provides a necessary caching benefit
- The expression involves a side effect such as a database call, network request, or mutation, where duplicate evaluation would change program behavior
- The variable name captures domain intent that the raw expression does not convey — for example, `isVipCustomer` is more meaningful than the inline Boolean expression it wraps

## Trade-offs
Inlining a temporary variable reduces the number of moving parts in a method, which generally improves maintainability by eliminating a name that must be kept in sync with its purpose. In languages with optimizing compilers, there is no runtime cost to either form. However, if the expression is costly and was previously evaluated once, inlining causes it to be evaluated at every reference site, which can degrade performance in tight loops or expensive I/O operations. The readability trade-off hinges on expression complexity: a short expression like `order.total()` is clearer inline, while a multi-term computation like `(basePrice * discountRate) + shippingCost - loyaltyCredit` may benefit from a named variable even if used only once. Developers should evaluate each case on whether the variable name adds information beyond what the expression itself conveys.

## Connections
Inline Temp is the inverse of Extract Variable and is frequently used as a stepping stone toward Replace Temp with Query or Extract Method. By removing the temporary variable, the expression becomes visible at the call site, making it easier to recognize patterns that warrant extraction into a dedicated method. This refactoring can help address the Comments smell indirectly: when a variable that once needed an explanatory comment is inlined into a context where its purpose is obvious, the comment becomes unnecessary and can be removed.

---

*Based on: Refactoring (Fowler, 1999)*
