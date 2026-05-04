# Introduce Foreign Method

## Motivation

Introduce Foreign Method provides a localized workaround when a library or framework class lacks behavior that your domain requires and you cannot modify the source. The classic scenario: a date utility from the standard library has no method to compute the next business day, and your code needs this computation in several places. Rather than duplicating the calculation at each call site, you concentrate it in a single static method on a client class, passing the library object as a parameter. The method acts as if it belongs to the foreign class — operating on its data, using its public interface — while living outside it.

This technique is a tactical compromise, not an architectural solution. It acknowledges that modifying the library is impossible and wraps the missing behavior in the nearest convenient location. The method is "foreign" because its natural home is the class it operates on, not the class that hosts it.

## Mechanics

1. Create a new static method in the client class that uses the foreign class most heavily. The method must accept an instance of the foreign class as its first parameter, enabling it to operate on that object's state through its public interface.

2. Extract the duplicated logic from each call site into this new method. Ensure the method is self-contained: it should not access instance state of the client class beyond what is passed as arguments.

3. Replace each duplicated code fragment with a call to the foreign method. Verify that the call sites produce identical results after the replacement.

4. Annotate the method with a comment marking it as a foreign method and naming the class it conceptually extends. This annotation serves as a contract: if the library ever comes under your control, this method should migrate to its rightful owner. Include the library version or a note about the unfulfilled need to guide future maintainers.

## Indications

**Signs suggesting this refactoring:**
- Incomplete Library Class: a third-party or framework class lacks a method that multiple call sites in your codebase require
- The same computation involving the library object's data appears in more than one place, creating duplication
- The missing functionality is a single method or a small cohesive set — not a broad extension of the library's domain

**When to avoid:**
- Multiple foreign methods have accumulated on the same library class, indicating that Introduce Local Extension would provide a cleaner organizational structure
- The missing behavior is substantial enough to warrant a full abstraction layer rather than scattered static helpers
- The library class is under your team's control and can be modified directly — modify it instead

## Trade-offs

Introduce Foreign Method trades structural purity for pragmatism. The method sits in a class that has no domain claim on the behavior, which can puzzle future readers who encounter a date computation living inside a report generator. The foreign-method annotation mitigates this confusion but cannot eliminate it entirely. On the positive side, the technique is low-cost: no new classes, no inheritance hierarchies, no wrapper plumbing. A single static method with a clear parameter contract is the lightest possible extension mechanism.

The technique does not scale well. When one foreign method becomes three, then seven, the client class accumulates unrelated static helpers that blur its own responsibility. At that threshold, Introduce Local Extension becomes the appropriate upgrade path: either a subclass that inherits the library's interface and adds the missing methods, or a wrapper that delegates to the library instance while presenting the extended API. The foreign method is a seed that, once it germinates into multiple methods, should be transplanted into a proper extension class.

## Connections

Introduce Foreign Method is the lighter companion to Introduce Local Extension — use the foreign method for one-off additions and upgrade to a local extension when the method count grows. Both address the Incomplete Library Class smell. The technique shares intent with the Adapter Pattern, though adapters translate between interfaces while foreign methods add missing behavior. The Strategy Pattern can also appear in this context: rather than extending the library class, a strategy object encapsulates the variant behavior and accepts the library object as input. Extract Method is often a precursor: duplicated logic is first extracted into a local method, then recognized as a foreign method candidate and marked accordingly.

---

*Based on: Refactoring (Fowler, 1999)*
