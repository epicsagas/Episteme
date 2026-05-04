# Decompose Conditional

## Motivation

Decompose Conditional breaks a complex conditional statement into clearly named method calls for the condition itself, the consequent body, and the alternative body. Long `if-else` blocks that mix business logic with low-level comparisons force readers to hold two mental models simultaneously: what the condition checks and what each branch does. As the branch logic grows, comprehension degrades because the condition's intent fades from memory while the reader works through the `then` clause, and by the time they reach the `else` clause, the original condition's context has been lost entirely. Named methods solve this by replacing the raw boolean expression and branch bodies with self-documenting calls that state their purpose in plain language. The pattern appears frequently in pricing engines, date-sensitive calculations, and any domain where conditional logic encodes nuanced business rules.

## Mechanics

1. Identify the conditional expression in the `if` statement. Extract it into a dedicated boolean method using Extract Method. Choose a name that describes the domain concept, such as `isSummer(date)` rather than `isWithinRange`.
2. Extract the body of the `then` clause into its own method, naming it to reflect the action taken when the condition holds. For example, `charge = quantity * summerRate` becomes `charge = summerCharge(quantity)`.
3. Extract the body of the `else` clause into a separate method following the same naming discipline, such as `winterCharge(quantity)`.
4. If the original conditional used temporary variables shared between the condition and the branches, pass them as parameters to the extracted methods or convert them to queries on the relevant object.
5. Verify that each extracted method is self-contained and does not rely on mutable state shared with its siblings. If coupling remains, consider Introduce Parameter Object to bundle related values.
6. Run tests after each extraction to confirm that the refactored conditional produces the same results as the original monolithic version.

## Indications

**Signs suggesting this refactoring:**
- A conditional whose boolean expression spans multiple lines or combines several comparisons
- `if-else` blocks where the branch bodies each exceed a few lines and encode distinct domain operations
- Readers who need to scroll or re-read the condition after finishing a branch to recall what triggered it
- Conditional logic that requires an explanatory comment to convey its purpose

**When to avoid:**
- The condition and branches are already short and self-explanatory, making extraction merely bureaucratic
- The conditional is a simple null check or type dispatch where a method name would add no information beyond the expression itself
- Performance-critical paths where the method-call overhead, though typically negligible, is demonstrably measurable

## Trade-offs

Decomposing a conditional replaces opaque inline logic with a readable narrative: `if (isSummer(date)) { charge = summerCharge(quantity); } else { charge = winterCharge(quantity); }`. The primary benefit is communicative clarity: each component names its intent, and the conditional reads like a sentence. However, the refactoring disperses logic across multiple methods, which can make debugging harder when a developer must jump between files to trace the full flow. In small codebases or simple conditionals, the indirection costs more than the clarity gains. The extracted methods also become part of the class's public interface unless carefully scoped, potentially exposing implementation details. Apply this refactoring when the conditional encodes a non-trivial business rule that benefits from a named, testable abstraction.

## Connections

This technique directly combats the Long Method smell by distributing conditional logic across focused, named methods. It is built on Extract Method, applying that foundational refactoring three times: once for the condition and once for each branch. Consolidate Conditional Expression operates at a different level, merging multiple separate conditionals rather than decomposing one. Replace Conditional with Polymorphism can follow this refactoring when the decomposed conditional varies by type and would benefit from dynamic dispatch. Introduce Assertion often complements decomposition by enforcing preconditions that the extracted condition method assumes to be true.

---

*Based on: Refactoring (Fowler, 1999)*
