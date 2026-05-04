# Remove Setting Method

## Motivation

Remove Setting Method eliminates a setter to prevent modification of a field after object construction, locking the value for the object's remaining lifetime. When a field's value is established once at creation and should never change, exposing a setter creates a mismatch between the intended semantics and the actual API. Any caller can invoke the setter at any point, potentially violating invariants that the rest of the class assumes hold permanently.

This refactoring is a practical step toward immutability. An object whose critical fields can no longer be reassigned is easier to reason about in concurrent contexts, safer to share across boundaries, and simpler to cache or compare by value. The technique is particularly relevant when applying Change Reference to Value, where an object must behave as a value whose identity is defined entirely by its immutable state.

## Mechanics

1. Verify that the field's value is genuinely stable after construction. Search for all setter invocations and confirm that each occurs either inside the constructor itself or immediately after construction at the creation site.
2. For each setter call that sits right after a constructor invocation, move the argument into the constructor's parameter list so the value is supplied during initialization. Update the constructor to assign the field directly.
3. Replace any setter calls inside the constructor body with direct field assignments, removing the indirection through the setter method.
4. Delete the setter method. Run the test suite to confirm that no remaining code attempts to call it.

## Indications

**Signs suggesting this refactoring:**
- A setter is called only in one place: immediately after the object is constructed, before it is used for anything else.
- The domain model treats the field as an intrinsic, unchangeable property of the object, such as a date of birth or a product identifier.
- Tests or comments explicitly assert that the field must not change after initialization.

**When to avoid:**
- The field legitimately changes during the object's lifetime as part of normal business operations, such as a status or last-modified timestamp.
- A framework requires a public setter for deserialization or dependency injection, and the framework cannot be configured to use constructor injection.
- The setter is part of a published interface used by external clients who rely on the ability to update the field.

## Trade-offs

Immutability delivers strong guarantees: once a field is locked at construction, no subsequent method call can corrupt it, and no thread can observe a partially constructed or mutated state. This simplifies reasoning about the object and eliminates entire categories of bugs related to unexpected state transitions. The cost is reduced flexibility. If a future requirement demands that the field become mutable, the setter must be reintroduced and all call sites reexamined for correct ordering. The refactoring also shifts responsibility to the constructor, which may acquire more parameters and require more careful setup at each creation site. When construction is complex, a Builder pattern or a factory method can absorb that complexity while still preventing post-construction mutation.

## Connections

Remove Setting Method is a key enabler of Change Reference to Value, where an object's identity is defined by its immutable fields rather than its memory address. It supports the broader move toward Value Objects, which rely on fields being set once and never altered. The technique frequently follows Encapsulate Field, which initially introduced the setter; Remove Setting Method then narrows access further. Replace Constructor with Factory Method can complement it by hiding the now-heavier constructor behind a descriptive creation method. On the smell side, removing unnecessary setters addresses the Temporary Field smell that arises when fields are set in some code paths but not others.

---

*Based on: Refactoring (Fowler, 1999)*
