# Encapsulate Collection

## Motivation

A collection field exposed through a simple getter grants callers direct access to the internal container. Callers can add, remove, or replace elements without the owning class ever knowing, which breaks encapsulation and makes it impossible for the owner to enforce invariants such as "every element must be non-null" or "removing an element should update the aggregate count." A raw setter that accepts an entirely new collection is equally dangerous because it bypasses any validation the owner might perform.

Proper encapsulation of a collection means the owner controls every modification. The getter returns an unmodifiable view or a defensive copy, and mutations happen exclusively through dedicated add and remove methods that can validate inputs, fire events, or maintain derived state. This discipline keeps the owner in full command of its own data.

## Mechanics

1. Add dedicated methods for adding and removing a single element. Each method should accept the element as a parameter and perform any validation or side-effect work the owner requires.
2. Ensure the collection field is initialized to an empty container in the constructor. This prevents null-pointer errors and guarantees that add and remove methods always have a valid target.
3. Replace every call site that assigns a whole collection through the setter with calls to the new add and remove methods. If bulk replacement is genuinely needed, rename the setter to something like `replaceAll` to make the destructive nature of the operation explicit.
4. Change the getter to return an unmodifiable view of the collection. In languages that support read-only wrappers natively, use them; otherwise, return a shallow copy. This prevents callers from mutating the collection while still allowing them to read its contents.
5. Scan client code for patterns such as iterating over the collection to compute aggregates or filter elements. Consider whether these operations belong as methods on the owning class instead, further reducing the need for callers to access the raw collection.

## Indications

**Signs suggesting this refactoring:**
- A getter returns the collection object itself, and callers directly call add, remove, or clear on it, bypassing the owner's awareness.
- The owning class has derived fields or counters that fall out of sync because external code modified the collection without going through the owner.
- A setter accepts a new collection wholesale, and callers replace the entire contents without triggering any of the owner's validation logic.

**When to avoid:**
- The collection is purely internal, never exposed outside the class, and no invariant depends on its contents. Encapsulation would add indirection with no practical benefit.
- Performance profiling shows that defensive copying in the getter creates measurable overhead in a hot path, and no caller ever modifies the returned collection.

## Trade-offs

Encapsulating a collection strengthens the owning class's control over its invariants, but it introduces a more verbose API. Callers accustomed to direct collection manipulation must learn new methods, and bulk operations become less convenient unless the owner provides explicit support for them. Returning a defensive copy on every getter call is safe but allocates memory on each invocation, which can matter in tight loops. Returning an unmodifiable view avoids the allocation but can surprise callers who attempt mutations and receive a runtime exception. The right balance depends on how widely the collection is shared and how critical its invariants are.

## Connections

This refactoring frequently targets the Data Class smell, where a class exposes its fields without protecting them. It pairs naturally with Encapsulate Field, which provides the same discipline for scalar fields. The Observer pattern benefits from collection encapsulation because the owner can notify subscribers whenever the collection changes. Move Method may be needed afterward to pull collection-centric operations out of client code and into the owning class, further tightening encapsulation.

---

*Based on: Refactoring (Fowler, 1999)*
