# Encapsulate Field

## Motivation

A public field allows any code in the program to read or write it directly, bypassing any validation, logging, or derived-state updates the owning class might require. This open access violates one of the foundational principles of object-oriented design: an object should manage its own state and present a controlled interface to the outside world. Without encapsulation, a change to the field's semantics ripples across every call site rather than being confined to the owning class.

Encapsulation converts the public field into a private one guarded by getter and setter methods. These methods become the single choke point through which all access flows, enabling the owner to validate inputs, fire change notifications, compute derived values lazily, or replace the underlying storage entirely without affecting callers.

## Mechanics

1. Create a getter method that returns the field's current value and a setter method that assigns a new value. Initially keep the field public so existing direct-access call sites continue to compile.
2. Search for every location that reads the field and replace it with a call to the getter. Search for every location that writes the field and replace it with a call to the setter.
3. Once all direct references have been redirected through the accessors, reduce the field's visibility to private. Run the full test suite to confirm that no direct access was missed.
4. Inspect the setter for opportunities to embed validation logic, and the getter for opportunities to compute a derived value or implement lazy initialization. This is where the real payoff of encapsulation emerges.

## Indications

**Signs suggesting this refactoring:**
- A field is declared public or package-private, and external classes read or mutate it without the owner's knowledge.
- Adding a constraint to the field, such as a range check or a format requirement, would require auditing every call site instead of modifying a single setter.
- A future change to the field's representation, such as replacing a stored value with a computed one, would break callers that rely on direct access.

**When to avoid:**
- In performance-critical inner loops where the overhead of a method call is measurable and the field carries no invariant that requires protection. Such cases are rare and should be confirmed with profiling before bypassing encapsulation.
- In value-object-like data structures where the fields are trivial and no behavior depends on them, such as a simple coordinate pair used only for transport between layers.

## Trade-offs

Encapsulation adds a layer of indirection between the data and its consumers, which marginally increases code volume and can obscure the simplicity of a plain data field. In small, self-contained programs this overhead may feel like ceremony without benefit. As programs grow, however, the single point of control that accessors provide becomes invaluable: validation, logging, thread-safety guards, and lazy computation all find a natural home in the getter or setter without disturbing any caller. The cost is modest at the scale of a single field, but applying it uniformly across a large codebase can generate hundreds of trivial accessor methods. Judicious application, guided by whether the field has invariants to protect or is likely to change representation, keeps the benefit-to-noise ratio high.

## Connections

Self Encapsulate Field is the internal counterpart, where a class accesses its own fields through getters and setters rather than directly. Encapsulate Collection extends this principle to collection-typed fields with additional safeguards. This refactoring is often the first step before Move Method, because once the field is encapsulated, behavior that depends on it can be relocated more easily. It addresses the Data Class smell by giving a passive data holder the ability to enforce constraints. The Template Method pattern relies on encapsulated fields to let subclasses override access behavior without altering the class's public contract.

---

*Based on: Refactoring (Fowler, 1999)*
