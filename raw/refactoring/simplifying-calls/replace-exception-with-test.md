# Replace Exception with Test

## Motivation

Replace Exception with Test converts a try-catch block that handles a predictable, checkable condition into a straightforward conditional test placed before the operation that would trigger the exception. The core insight is that exceptions should signal truly unexpected failures, not control flow for conditions the code can foresee and validate. Using exception handling for routine boundary checks, such as testing whether an array index is in bounds before accessing it, conflates error recovery with normal branching logic and obscures the programmer's actual intent.

This refactoring is prompted whenever the catch block implements logic that belongs in the main flow. An array-access method that catches IndexOutOfBoundsException and returns a default value is using the exception mechanism as a disguised conditional. A pre-access bounds check expresses the same intent more directly, performs better because it avoids the cost of stack unwinding, and is easier for readers to understand because the guard condition sits visibly before the guarded operation.

## Mechanics

1. Identify the condition that triggers the exception and formulate an equivalent boolean test that can be evaluated before the risky operation. Ensure the test covers exactly the same cases that the exception would catch.
2. Place the test in an if-statement preceding the try block. Move the logic from the catch clause into the if-branch, preserving identical behavior.
3. Modify the catch block to throw an assertion error or a more specific unexpected exception. This serves as a safety net during the transition: if the test misses a case, the failure is still caught.
4. Run the full test suite. If no tests trigger the modified catch block, the test is proven complete and the entire try-catch structure can be removed, leaving only the conditional guard and the happy-path code.

## Indications

**Signs suggesting this refactoring:**
- A catch block handles an exception that corresponds to a condition easily verified before the operation, such as index bounds, null references, or division by zero.
- The catch body contains logic that is part of normal control flow, like returning a default value, rather than genuine recovery or logging.
- The exception is caught and handled locally every time, never propagating up the call stack, which indicates it is expected rather than exceptional.

**When to avoid:**
- The condition is genuinely unpredictable or depends on external state that cannot be tested reliably, such as a network timeout or a file-system race condition.
- The pre-check would duplicate complex logic that the underlying operation already performs internally, introducing the risk of the check and the operation diverging over time.
- The exception comes from a third-party library whose internal validation logic is not fully known or may change between versions.

## Trade-offs

A conditional test placed before the operation makes the guard condition explicit and the code's intent transparent. It eliminates the performance overhead of exception construction and stack unwinding, which matters in tight loops or high-frequency code paths. The result is code that reads top-to-bottom without hidden control-flow jumps. The risk is that the test may not perfectly replicate the exception's trigger condition. If the check is slightly different from what the underlying operation validates, the refactoring introduces a subtle gap where the old catch would have handled an edge case but the new test does not. This is particularly dangerous with operations whose validation rules evolve independently. In those cases, the exception handler acts as a broader safety net that the conditional test cannot fully replace. The technique is best applied when the condition is simple, well-defined, and unlikely to change.

## Connections

Replace Exception with Test is the inverse of Replace Error Code with Exception. Both address how a program communicates failure, but they move in opposite directions along the predictability spectrum. The technique supports the Command and Query Separation principle by ensuring that queries perform straightforward checks rather than relying on exception machinery for expected outcomes. It relates to the Null Object pattern in cases where the catch body returns a default value: instead of catching an exception and substituting a default, the code can test for the condition and return the default directly, or use a Null Object that provides the same behavior without branching. Guard clauses and the Early Return pattern are close allies, as the extracted test often becomes a guard clause at the top of the method.

---

*Based on: Refactoring (Fowler, 1999)*
