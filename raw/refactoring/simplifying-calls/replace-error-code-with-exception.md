# Replace Error Code with Exception

## Motivation

Replace Error Code with Exception converts a method that signals failure through a special return value into one that throws an exception, forcing the caller to acknowledge and handle the error path explicitly. Returning sentinel values like negative one, null, or error-code enums is a procedural relic that scatters error-checking logic across every call site. Each caller must remember to test the return value and branch accordingly, and a single missed check silently propagates an invalid state downstream.

Exceptions invert this responsibility. The language runtime ensures that an unhandled failure does not go unnoticed; it unwinds the call stack until a handler is found or the program terminates with a clear diagnostic. This model is especially important for operations that can fail in ways the caller cannot predict or recover from locally, such as insufficient funds during a withdrawal or a missing resource during file access. Exceptions also work in constructors, which have no return value to encode an error code into.

## Mechanics

1. Identify every call site that checks the method's return value for an error sentinel. Wrap each call in a try-catch block that handles the forthcoming exception type, preserving the existing error-recovery logic inside the catch clause.
2. Inside the method, replace every statement that returns an error code with a throw statement that raises an appropriate exception class. Carry any relevant diagnostic information, such as the requested amount or the current balance, as fields on the exception.
3. Change the method's return type to void if it previously returned only success-or-error codes, or to the genuine result type if the return value served a dual purpose.
4. Document the thrown exception in the method's signature annotation or doc comment so that callers can discover the contract without reading the body.

## Indications

**Signs suggesting this refactoring:**
- A method returns a magic number or enum value to indicate failure, and callers must check the return before proceeding.
- The same error-checking pattern, such as `if (result == -1)`, repeats at every call site, creating Shotgun Surgery risk when the error code changes.
- A constructor or void method needs to report a failure but has no return value to encode it in.
- The error condition is exceptional rather than expected: it represents a violation of a precondition rather than a normal branching case.

**When to avoid:**
- The condition is an expected, frequent outcome rather than an exceptional one, such as a lookup that legitimately may find nothing. In that case, returning a sentinel or an Optional is more honest than throwing.
- The method is performance-critical and the exception-throwing overhead, including stack capture, is unacceptable.
- The codebase runs in an environment where exceptions are disabled or culturally discouraged, such as certain embedded or real-time systems.

## Trade-offs

Exceptions centralize error handling and make the failure path visible through type signatures and language-enforced control flow. They eliminate the need for every caller to implement manual error-code checking, which reduces the chance of a missed check leading to silent corruption. The trade-off is that exceptions introduce non-local control flow: understanding what happens after a throw requires tracing up the call stack to the nearest handler. This can make debugging harder if exceptions are caught too broadly or too far from the source. Exception abuse, where exceptions replace normal conditional logic, creates code that is harder to follow than straightforward conditionals. Performance is another consideration: exception construction captures the stack trace, which is expensive relative to a simple return value. In hot loops or high-throughput paths, the overhead matters. Finally, checked exceptions in languages like Java force every intermediate caller to declare the exception, which can clutter signatures throughout a call chain.

## Connections

Replace Error Code with Exception is the inverse of Replace Exception with Test, which moves in the opposite direction when an exception guards a condition that a simple check could handle. The technique supports the Command and Query Separation principle by allowing queries to return values and commands to signal failure through exceptions rather than mixing the two concerns in a return value. It pairs with Separate Query from Modifier when a method both performs an action and returns a result, and the error path needs to be disentangled. On the design-pattern side, the resulting exception classes can evolve into a hierarchy that supports specialized handling, aligning with the Null Object pattern for cases where a missing value should be treated as a valid alternative rather than an error.

---

*Based on: Refactoring (Fowler, 1999)*
