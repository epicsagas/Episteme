# Change Unidirectional Association to Bidirectional

## Motivation

A unidirectional association allows one class to reference another but provides no path in the reverse direction. This asymmetry becomes a liability when client code repeatedly needs to answer questions like "which parent owns this child?" or "which orders contain this product?" Without a back-pointer, callers must perform scans, maintain auxiliary lookup tables, or pass context through deep call chains, all of which scatter navigation logic across the codebase and degrade performance.

Adding the reverse link makes the relationship explicit and navigable from both sides. This is especially valuable in domain models where the association carries business meaning in both directions, such as an employee who must know their department and a department that must enumerate its staff.

## Mechanics

1. Add a field to the class that lacks the reverse reference. Choose a name that clearly conveys the relationship's semantics from this new direction.
2. Decide which class owns the association and will be responsible for keeping both directions consistent. The owning class is typically the one with the stronger lifecycle dependency or the one whose creation naturally establishes the link.
3. In the owning class, create a method that sets the forward reference and simultaneously updates the reverse reference in the target object. This method becomes the single authority for establishing the link, preventing partial updates.
4. In the non-owning class, provide a package-private or internal helper that the owning class calls to maintain the back-pointer. Restrict its visibility so external code cannot corrupt the association by updating only one side.
5. Audit existing constructors and setters that already establish the forward link, augmenting them to also set the reverse reference through the new helper methods. Remove any code that previously computed the reverse path indirectly.

## Indications

**Signs suggesting this refactoring:**
- Callers frequently traverse the association in the unsupported direction, often by iterating through collections or querying external stores to recover information that a back-pointer would provide instantly.
- A method receives an object as a parameter only because the caller had to look it up, and the callee already holds a reference to the other end of the association.

**When to avoid:**
- The reverse direction is rarely needed and a simple query method or lookup table can supply it without adding persistent state.
- The two classes belong to different modules or layers where a mutual dependency would create a circular import or violate a layering constraint.

## Trade-offs

Bidirectional associations trade simplicity for convenience. Maintaining two pointers instead of one doubles the synchronization burden: every creation, reassignment, and deletion must update both sides atomically. This extra bookkeeping introduces opportunities for inconsistency, especially when objects are removed from collections without cleaning up their back-pointers. On the other hand, the performance benefit of constant-time reverse navigation can be decisive in traversal-heavy domains. Before committing to a bidirectional link, verify that the reverse direction is used often enough to justify the added maintenance cost. If the need is sporadic, a dedicated query method may deliver the same capability with less coupling.

## Connections

The inverse operation is Change Bidirectional Association to Unidirectional. This refactoring frequently accompanies Extract Class, where a newly extracted class needs a reference back to its origin. It can trigger the Inappropriate Intimacy smell if both classes begin reaching deeply into each other's state, which in turn may call for Move Method to redistribute responsibilities. Patterns such as Observer and Mediator can sometimes replace a direct bidirectional link with event-based notification, reducing coupling at the cost of indirection.

---

*Based on: Refactoring (Fowler, 1999)*
