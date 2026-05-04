# Change Reference to Value

## Motivation

A reference object is one whose identity matters: two variables pointing to the same instance reflect each other's changes. This identity tracking is valuable for entities such as customers or orders, but it becomes unnecessary overhead for small, stable data carriers like currency amounts, date ranges, or geographic coordinates. When every access to such an object requires fetching it from a central registry and managing its lifecycle, the design imposes friction disproportionate to the object's simplicity. Converting a reference object into a value object eliminates the need for centralized storage, factory lookup, and identity synchronization.

Value objects are defined by their attributes rather than their identity. Two value objects with equivalent content are considered interchangeable, which makes equality comparison straightforward and eliminates whole categories of aliasing bugs. This shift is especially beneficial in concurrent and distributed environments where sharing mutable references across threads or processes introduces complex synchronization requirements.

## Mechanics

1. Ensure the object is immutable. Remove all mutating methods and make every field final or read-only. All state should be established exclusively through the constructor.
2. Implement an equality method that compares the significant fields of two instances. In languages that support operator overloading, consider overriding the equality operator as well. This step is essential because value semantics require that two distinct instances with identical content be treated as equal.
3. Simplify the construction pathway. If a factory method was introduced to enforce single-instance semantics, replace its callers with direct constructor invocations. Make the constructor public unless there is a remaining reason to restrict creation.

## Indications

**Signs suggesting this refactoring:**
- The object's data never changes after construction, yet the codebase retrieves instances from a registry or cache as though updates must propagate.
- Distributed or parallel subsystems struggle with shared mutable state, and the object in question is a simple data carrier such as a monetary amount or a measurement.
- The overhead of maintaining a central store for the object outweighs the benefit of shared identity.

**When to avoid:**
- The object's state mutates and those mutations must be visible to all holders of a reference. Switching to a value object would require callers to manually propagate updates, reintroducing the very complexity the refactoring aims to remove.
- The domain distinguishes between objects that happen to carry the same data but represent different entities, such as two people with the same name.

## Trade-offs

Value objects are simpler to construct, serialize, and reason about, but they sacrifice identity-based sharing. If a value appears in many places and its data needs to change, every holder must receive a new copy rather than seeing the change automatically through a shared reference. This propagation cost can negate the simplification gains in domains where updates are frequent. Immutability also means that any transformation produces a new instance, which can pressure garbage collection in hot loops. Weigh the frequency of mutation against the overhead of reference management: rarely changed data favors value semantics, while frequently updated shared state favors reference semantics.

## Connections

The inverse transformation is Change Value to Reference. This refactoring often follows Replace Data Value with Object once the extracted object stabilizes and no longer requires shared identity. It addresses the Data Class smell by encouraging immutability and meaningful equality. Design patterns such as Flyweight rely on value-like equality to share instances safely, and Functional Programming paradigms treat all data as values by default.

---

*Based on: Refactoring (Fowler, 1999)*
