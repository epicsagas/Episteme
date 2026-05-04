# Inline Method

## Motivation
Inline Method removes a delegation layer that has stopped carrying its weight. A method that simply forwards to another method, wraps a single expression, or exists only because a developer once anticipated future complexity — but that complexity never arrived — adds navigation overhead without aiding comprehension. When reading the caller requires jumping to the callee only to find a trivial body, the abstraction is obscuring meaning rather than revealing it. Inlining collapses the indirection so the reader sees the complete logic in one place.

This refactoring is also common during architectural simplification. Codebases accumulate thin utility methods during feature development, and subsequent refactorings like Extract Method often absorb their logic into better-named new methods, leaving the originals as hollow wrappers. Before performing a larger restructuring, inlining these vestigial methods reduces the surface area that needs to be tracked and ensures no hidden polymorphic dispatch survives the reorganization.

## Mechanics
1. Confirm the method is not overridden in any subclass, since inlining would eliminate the polymorphic dispatch point and silently change behavior.
2. Verify that the method is not referenced as a callback, event handler, or reflection target where removing it would break callers that depend on its identity.
3. Find every call site and replace the call with a copy of the method body, adjusting variable names and `this` references to match the calling context.
4. After all call sites have been converted, delete the original method declaration and run the test suite to confirm behavioral equivalence.
5. If the method body references private members of its declaring class that are inaccessible at the call site, widen access to internal or package-private rather than public, or reconsider whether inlining is appropriate.

## Indications

**Signs suggesting this refactoring:**
- A method body consists of a single return statement delegating to another method or accessing a single field
- The method name provides no more information than reading the body directly would
- A previous refactoring (such as Replace Temp with Query or Extract Method) has made the method's purpose redundant
- The method exists solely to satisfy an anticipated interface that was never realized

**When to avoid:**
- The method is part of a public API or published interface where callers depend on the method signature
- Subclasses override the method, making it a legitimate polymorphism point
- The method serves as a seam for testing — test subclasses or mocks may rely on it as an override target
- The method body is complex enough that the name genuinely saves cognitive effort at the call site

## Trade-offs
Inlining reduces indirection and makes the call site self-contained, which improves readability when the method body is shorter than its name. It also eliminates one method from the class, shrinking the overall API surface that developers must learn. The downside is that inlining duplicates logic if the method is called from multiple places, though this is rarely the case for the trivial methods that are candidates for inlining. A more subtle risk is losing a well-named abstraction: a method called `isEligibleForRefund` communicates intent better than the inline expression `order.total() > threshold && !order.hasFlag(FRAUD_REVIEW)`. Developers should inline only when the body is genuinely more informative than the name, not merely when the body is short.

## Connections
Inline Method is the inverse of Extract Method; the two form a pair that developers cycle between as abstractions evolve. It directly addresses the Speculative Generality smell, where methods were created for future flexibility that never materialized. Inlining is often a preparatory step before applying Substitute Algorithm: collapsing a delegation chain into a single method makes the algorithm boundary explicit and easier to replace wholesale. When a thin method wraps a temporary variable access, Inline Temp may be applicable alongside or instead of this refactoring.

---

*Based on: Refactoring (Fowler, 1999)*
