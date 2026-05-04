# Replace Temp with Query

## Motivation
Replace Temp with Query converts a local variable that holds a computed value into a method that returns the same value on demand. The motivation arises when the same computation appears in multiple methods of the same class — each one declaring its own temporary variable to store the result — or when the temporary variable's existence prevents Extract Method from working cleanly because the variable would need to be passed as a parameter to every extracted sub-method. By promoting the expression into a query method, the computation becomes a shared, named operation accessible throughout the class and, if made public, throughout the package.

The query method also serves as an authoritative single source of truth for the computation. When business rules change — say, the base price formula now includes a surcharge — the developer updates the query method once rather than hunting through every method that declares a local `basePrice` variable. This centralization eliminates the Duplicate Code smell at its root and ensures consistency across all callers.

## Mechanics
1. Ensure the temporary variable is assigned exactly once within the method; if it receives multiple assignments, apply Split Temporary Variable first to isolate each distinct use.
2. Confirm the right-hand side of the assignment is a pure expression with no side effects, since the resulting query method may be called multiple times.
3. Extract the expression into a new method — typically named after the value it computes, such as `basePrice` or `discountedTotal` — and verify the method returns the correct type.
4. Replace the temporary variable assignment with a call to the new query method, then replace all subsequent references to the variable with the same call.
5. Remove the temporary variable declaration and run tests to confirm behavioral equivalence, paying attention to performance-sensitive sections where repeated evaluation may matter.

## Indications

**Signs suggesting this refactoring:**
- The same computed expression appears as a temporary variable in two or more methods within the same class
- A temporary variable's scope is the entire method, and its computed value would be meaningful to other methods on the same object
- An attempt to apply Extract Method is blocked because the temporary variable would need to be threaded through every extracted sub-method as a parameter

**When to avoid:**
- The expression is computationally expensive and the temporary variable legitimately caches the result to avoid repeated evaluation in a tight loop or performance-critical path
- The expression produces side effects — database queries, network calls, mutations — that should not be invoked multiple times transparently
- The temporary variable is reassigned conditionally, making it state-dependent rather than a pure computation that can be freely reevaluated

## Trade-offs
The primary gain is reuse: a query method can be called from any method in the class without duplicating the computation logic, and it can be promoted to a public accessor if other classes need the same value. The resulting code is also more amenable to further refactoring, particularly Extract Method, because the query eliminates local-state dependencies. The cost is potential performance degradation if the query is called frequently in a hot path and the original temporary variable was providing a caching benefit. In practice, modern compilers and JIT runtimes often inline short query methods, making the overhead negligible. When performance is genuinely a concern, developers can memoize the query result or revert to a cached local variable in the critical path while keeping the query method available for less demanding callers.

## Connections
Replace Temp with Query is a natural companion to Extract Method: removing temporary variables as shared local state clears the path for extracting sub-methods without ballooning parameter lists. It addresses the Duplicate Code smell by centralizing repeated computations into a single query method. The technique also supports the Template Method pattern, where abstract hook methods serve as queries that subclasses override to vary the computation. When the query becomes complex enough to require its own state, Replace Method with Method Object provides the next level of decomposition. The inverse direction — replacing a query call with a local temporary variable — is Inline Temp, which is sometimes appropriate when the query is called once and the expression is simple enough to read inline.

---

*Based on: Refactoring (Fowler, 1999)*
