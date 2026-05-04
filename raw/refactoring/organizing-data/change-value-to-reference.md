# Change Value to Reference

## Motivation

Value objects are created freely and compared by their content. This works well for data like dates or measurements, where two objects carrying the same value are effectively interchangeable. Problems arise when the same logical entity is represented by multiple distinct value instances scattered across the system. If one instance is updated, the others remain stale, forcing the codebase to either propagate changes manually or tolerate inconsistency.

Converting a value object into a reference object ensures that a single canonical instance represents each entity. Any change made through one reference is immediately visible to all other references, because they all point to the same memory location. This is essential for entities such as customers, accounts, or configuration entries where multiple subsystems must agree on the current state.

## Mechanics

1. Apply Replace Constructor with Factory Method to the target class. This centralizes instance creation into a single method that can later control whether a new object is allocated or an existing one is returned.
2. Decide whether the reference objects should be created eagerly at startup or lazily on first request. Eager creation simplifies reasoning but increases startup cost; lazy creation defers the expense but requires thread-safe initialization logic.
3. Inside the factory method, consult a registry or cache before constructing a new instance. If an object matching the requested key already exists, return it; otherwise, create one, store it, and return it. This registry can be a simple map keyed by a unique identifier.
4. Rename the factory method if its new semantics warrant clarification, for example from `createCustomer` to `getCustomer`, signaling that it may return a pre-existing instance rather than always allocating.

## Indications

**Signs suggesting this refactoring:**
- The same real-world entity is represented by multiple value objects, and updates to one instance do not propagate to others, causing stale-data bugs.
- The codebase is littered with manual synchronization logic, such as lookup-and-replace patterns, to keep duplicate instances in agreement.
- An entity that started as a simple data carrier has accumulated mutable state and now functions as a shared resource.

**When to avoid:**
- The object is genuinely immutable and carries no mutable state. Multiple copies of an immutable value are harmless, so the overhead of a reference registry is unnecessary.
- The domain has no natural unique key for the object, making it difficult to determine whether two instances represent the same entity.

## Trade-offs

Reference objects guarantee consistency across the system, but they introduce a centralized store that becomes a concurrency bottleneck and a single point of failure. Thread-safe access to the registry requires locks or concurrent data structures, adding non-trivial complexity. Reference identity also complicates serialization and deserialization, because reconstructing an object graph from a stream must reconnect references rather than create duplicates. In distributed systems, maintaining a single source of truth for each reference may require a distributed cache or database lookup, trading local speed for global consistency. Reserve this refactoring for entities where shared mutable state is a genuine requirement, not merely a convenience.

## Connections

The inverse transformation is Change Reference to Value. This refactoring commonly follows Replace Data Value with Object once the extracted object accumulates mutable state and shared identity becomes necessary. It relates to the Flyweight pattern, which also uses canonical instances to avoid duplication, though Flyweight focuses on memory efficiency rather than consistency. The Data Class smell may prompt this refactoring when a data holder evolves into a full-fledged domain entity.

---

*Based on: Refactoring (Fowler, 1999)*
