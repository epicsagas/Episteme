# Extract Variable

## Motivation
Extract Variable tackles code where a dense expression forces readers to mentally evaluate sub-terms before understanding the overall logic. Long Boolean conditions, chained arithmetic, deeply nested ternary operators, and compound string concatenations all qualify. When a developer feels the need to write an inline comment explaining what a sub-expression computes, that sub-expression is a strong candidate for its own named variable. The variable name replaces the comment with a living, compiler-verified label that stays synchronized with the code.

This technique is especially valuable in conditional branches where multiple terms combine with `&&` and `||`. Breaking each term into a boolean variable with a name like `isEligibleForUpgrade` or `hasExceededCreditLimit` makes the branch condition read like a business rule rather than a math puzzle. The approach also helps during debugging, because each intermediate value can be inspected independently in a debugger watch window.

## Mechanics
1. Identify a sub-expression within a larger statement whose meaning is not immediately obvious from the code alone.
2. Declare a new local variable initialized to that sub-expression, giving it a name that describes the value's purpose in domain terms.
3. Replace the original sub-expression with the variable reference, ensuring the expression type matches the variable type.
4. Repeat for each additional sub-expression that would benefit from a descriptive name, stopping when the remaining expression is self-explanatory.
5. Verify that the new variable is not reassigned later in the method; if it is, Split Temporary Variable may be needed first.

## Indications

**Signs suggesting this refactoring:**
- A conditional statement combines more than three terms, making the logic difficult to verify at a glance
- A calculation involves multiple arithmetic operators without parentheses clarifying evaluation order
- A developer writes an inline comment solely to explain what a sub-expression computes

**When to avoid:**
- The expression is already short and its meaning is clear from context, such as a single field access or a trivial two-operand arithmetic operation
- Extracting the variable would obscure short-circuit evaluation behavior that readers need to see inline, particularly in performance-sensitive guard clauses
- The sub-expression is used exactly once and a method-level extraction via Extract Method would provide better reusability

## Trade-offs
The immediate benefit is clarity: descriptive variable names encode intent directly into the code, eliminating the need for comments that drift out of sync. Debugging becomes easier because each intermediate result is materialized and inspectable. The cost is additional local variables that increase the method's variable count, which can feel noisy in very small methods. In languages with aggressive compiler optimization, the introduced variables have zero runtime cost, but developers should be aware that extracting parts of a compound Boolean may disable short-circuit evaluation if the extracted variable is computed eagerly before the condition is tested. When a variable name merely restates the expression in words — such as naming a sum `sumOfAAndB` — the extraction adds no value and should be skipped in favor of leaving the expression inline.

## Connections
Extract Variable is the inverse of Inline Temp, which removes an unnecessary intermediate variable by substituting the expression directly. It frequently accompanies Extract Method as a preparatory step: naming intermediate values inside a method makes it easier to decide which statements belong in the extracted method. The technique addresses the Comments smell by replacing explanatory prose with self-documenting variable names. When an extracted variable could be useful outside the current method, Replace Temp with Query generalizes the variable into a reusable query method. In performance-sensitive contexts, developers sometimes prefer Extract Method over Extract Variable because method boundaries enable caching and lazy evaluation strategies.

---

*Based on: Refactoring (Fowler, 1999)*
