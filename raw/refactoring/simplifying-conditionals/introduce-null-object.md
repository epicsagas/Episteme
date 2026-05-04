# Introduce Null Object

## Motivation

Introduce Null Object eliminates pervasive null checks by replacing null references with a special object that implements the same interface but provides do-nothing or default behavior. When a method returns null to signal the absence of a value, every caller must guard against it before proceeding. These guards accumulate across the codebase, inflating method length and scattering conditional logic that obscures the primary workflow. The null object pattern absorbs this complexity into a single class that encapsulates the "do nothing" behavior, allowing clients to interact with the object uniformly regardless of whether a real entity exists. This refactoring is most impactful in domains where optional collaborators are common: a customer without a subscription plan, an employee without an assigned manager, or a sensor that has not yet reported a reading.

## Mechanics

1. Identify the class whose instances may be null. Create a subclass of it that will serve as the null object. In languages that support interfaces, you may alternatively have both the real class and the null object implement a shared interface.
2. Add a query method such as `isNull()` to the base class, returning `false`. Override it in the null object subclass to return `true`. This method provides a migration path during incremental refactoring.
3. Review every method on the source class and implement a safe default in the null object for each one. For methods that return values, return a sensible default: zero for numbers, empty collections for lists, or another null object for reference types. For void methods, provide an empty body.
4. Locate every point in the codebase where the source class is instantiated or returned as null. Replace each `null` with an instance of the null object. Factory methods and dependency injection containers are common places to make this change.
5. Find every null comparison in the codebase involving the source class, such as `if (customer != null)`. Replace each with a call to `isNull()` or, ideally, remove the check entirely if the null object's default behavior makes the guard unnecessary.
6. Move conditional behavior that currently wraps null checks into the null object's method overrides. Where the real class performs meaningful work, the null object performs the no-op or default equivalent, achieving polymorphic dispatch.
7. Run the full test suite, paying special attention to scenarios where null was previously returned, to ensure the null object's defaults produce correct downstream results.

## Indications

**Signs suggesting this refactoring:**
- Frequent `if (x != null)` checks scattered across multiple callers of the same type
- Methods whose body is dominated by null-guard boilerplate rather than domain logic
- Repeated default-value logic that mirrors what a missing object should produce, such as returning zero charges for an unregistered customer
- Code that crashes with NullPointerException or similar errors because a null guard was missed in one of many call sites

**When to avoid:**
- The null case is genuinely exceptional and should be surfaced to the caller via an exception rather than silently handled
- Only one or two null checks exist, and introducing a class would be disproportionate overhead
- The object has many methods, most of which have no meaningful default, forcing the null object to throw UnsupportedOperationException for the majority of its interface
- The team's language or framework already provides an Optional or Maybe monad that achieves the same goal without a dedicated class

## Trade-offs

The null object pattern trades scattered conditional logic for a single class that centralizes default behavior. Callers become simpler because they no longer branch on null, and the risk of forgetting a null check is eliminated by construction. However, the refactoring introduces an additional type to the codebase and requires discipline to ensure that every new method added to the source class also receives an appropriate null implementation. Debugging can become more difficult because a silent no-op may mask a configuration error that would have been caught by a null pointer exception. The pattern also interacts subtly with equality semantics: two null objects of the same type are typically interchangeable, but identity comparisons or serialization logic may not treat them as equivalent. Apply this refactoring when null checks are numerous and the default behavior is well-defined and stable.

## Connections

Introduce Null Object directly addresses the Switch Statements smell when those switches check for null as a special case, and the Temporary Field smell when a field is null in some object states. It is closely related to Replace Conditional with Polymorphism, as both move conditional logic into polymorphic dispatch. The Null Object is itself a classic design pattern often paired with the Strategy pattern, where a null strategy provides the default behavior. Introduce Assertion can complement this refactoring by asserting that the null object has been properly initialized in places where complete elimination of the null check is not yet feasible. Special Case is Martin Fowler's preferred term for this pattern when the "null" object provides meaningful non-trivial defaults rather than mere no-ops.

---

*Based on: Refactoring (Fowler, 1999)*
