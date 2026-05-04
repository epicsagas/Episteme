# Change Bidirectional Association to Unidirectional

## Motivation

A bidirectional association exists when two classes hold mutual references to each other, enabling navigation in both directions. This structure becomes problematic when one direction goes unused, because the extra link adds maintenance burden without delivering value. Unnecessary bidirectionality forces both classes to coordinate updates whenever the relationship changes, creating synchronization bugs and making the object graph harder to reason about.

Memory management also suffers under mutual references. Garbage collectors that rely on reference counting or reachability analysis may retain objects that reference each other long after both become logically obsolete, leading to subtle memory leaks. In distributed systems, bidirectional links complicate serialization and increase the surface area for inconsistency between replicas.

## Mechanics

1. Survey all call sites that traverse the association from the direction you plan to remove. Confirm that none of these traversals are essential, or that an alternative retrieval strategy exists, such as a database query, a method parameter, or a lookup table.
2. For each call site that currently navigates the unwanted direction, introduce an alternative way to obtain the target object. Passing the object as a parameter to the method that needs it is often the cleanest substitute.
3. Remove any assignment logic that populates the reverse pointer. This includes constructor initialization, setter bodies, and any event-driven updates.
4. Delete the field holding the reverse reference and run the test suite to verify that no consumer was missed.

## Indications

**Signs suggesting this refactoring:**
- One side of a two-way link is never read or is read only in places where the object is already available through other means.
- The codebase contains awkward workarounds to keep both directions synchronized, such as notification methods that exist solely to update back-pointers.
- Memory profiling reveals objects surviving longer than expected because of mutual retention.

**When to avoid:**
- Both directions of the association are actively used in performance-critical paths where recomputing the reverse link would be expensive.
- The domain model genuinely requires mutual awareness, as in a parent-child relationship where both sides need constant access to each other.

## Trade-offs

Removing a direction reduces coupling and simplifies the classes involved, but it shifts the cost of obtaining the associated object onto the caller. If callers must now perform expensive queries or pass extra parameters through deep call chains, the net complexity may increase rather than decrease. Evaluate the frequency of reverse traversal against the maintenance cost of keeping the link bidirectional. In read-heavy domains where the reverse direction is queried occasionally, a unidirectional association combined with a targeted query method often wins. In write-heavy domains where both sides mutate in concert, the synchronization logic of a bidirectional link may be justified despite its complexity.

## Connections

The inverse operation is Change Unidirectional Association to Bidirectional. This refactoring directly addresses the Inappropriate Intimacy code smell, where two classes spend too much time delving into each other's private members. It also complements Extract Class by reducing the coupling that remains after a class has been split. Design patterns such as Observer can sometimes serve as a lighter-weight alternative to a full bidirectional link, providing one-way notification without a permanent reverse pointer.

---

*Based on: Refactoring (Fowler, 1999)*
