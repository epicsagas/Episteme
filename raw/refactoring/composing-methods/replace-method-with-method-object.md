# Replace Method with Method Object

## Motivation
Some methods grow so large and so entangled with local variables that Extract Method cannot be applied without passing an unwieldy number of parameters between the resulting pieces. The method may have ten or more local variables, each modified in multiple places, creating a web of dependencies that resists decomposition into smaller functions. In this situation, the method itself has become a complex algorithm whose internal state is too rich to express through a parameter list. Replace Method with Method Object resolves the impasse by promoting the entire method — along with all its local variables — into a dedicated class where the variables become fields and the algorithm can be broken into private methods that share state through the object rather than through arguments.

This refactoring is particularly relevant in domain-driven designs where a pricing engine, tax calculator, or simulation step naturally encapsulates a multi-stage computation. The resulting class often acquires a name that reflects the domain concept — `PriceCalculator`, `TaxComputation`, or `CollisionResolver` — which communicates more intent than a monolithic method called `compute` buried inside a domain entity.

## Mechanics
1. Create a new class whose name describes the algorithm's purpose in domain terms, not the mechanical act of decomposing a method.
2. Add a private field to hold a reference to the original object, since the algorithm likely needs access to its data and behavior.
3. Add a private field for each local variable in the original method, preserving their types and initial values in the constructor.
4. Write a constructor that accepts the original object reference and any initial values needed to seed the computation.
5. Copy the method body into a public method on the new class — often named `compute` or `execute` — replacing all local variable references with the corresponding field accesses.
6. Replace the original method body with a single line that instantiates the new class and calls its computation method, forwarding any required arguments.
7. With the algorithm now isolated in its own class, apply Extract Method freely to break the `compute` method into smaller private methods that share state through fields rather than parameters.

## Indications

**Signs suggesting this refactoring:**
- A method contains so many interdependent local variables that any attempt at Extract Method produces a parameter list longer than five arguments
- The method spans fifty lines or more and cannot be broken into meaningful sub-operations without passing a cluster of local variables between them
- The method represents a distinct domain computation — pricing, scheduling, scoring — that deserves its own identity beyond being a procedure on an entity class

**When to avoid:**
- The method is only moderately long and can be simplified through simpler refactorings like Extract Method, Replace Temp with Query, or Split Temporary Variable
- The overhead of introducing a new class outweighs the clarity gained, particularly in a small codebase where the extra class adds navigation complexity
- The language supports closures or inner functions that can capture local variables directly, providing a lighter-weight alternative to a full method object

## Trade-offs
The principal advantage is unlocking decomposition: once local variables become fields, any private method in the new class can read or write them without requiring explicit parameter passing, making it trivial to break a hundred-line method into ten focused methods. The new class also provides a natural home for related helper methods and constants, keeping the original class cleaner. The trade-off is increased structural complexity — an additional class in the codebase, an additional object allocation per invocation, and an additional level of indirection that readers must navigate. In performance-critical paths, the object allocation may matter, though modern garbage collectors make this concern negligible in most applications. Developers should also guard against turning the method object into a God Object by ensuring it remains focused on a single algorithm and does not accumulate unrelated responsibilities over time.

## Connections
Replace Method with Method Object is a last-resort enabler for Extract Method when local variable entanglement blocks direct extraction. It addresses the Long Method smell at scale, where simpler decompositions have failed. The resulting class often resembles the Command pattern, especially if the algorithm is later parameterized or queued for deferred execution. The technique can precede Introduce Parameter Object when the constructor parameter list grows large enough to warrant its own abstraction. In functional programming contexts, the same problem is addressed with closures or reader monads rather than dedicated classes, but the underlying motivation — sharing state across decomposed sub-functions without explicit parameter threading — is identical.

---

*Based on: Refactoring (Fowler, 1999)*
