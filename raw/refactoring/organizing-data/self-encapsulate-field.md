# Self Encapsulate Field

## Motivation

Direct field access within a class is simple and fast, but it tightly couples the class's internal methods to the field's current representation. If the field later needs to be computed lazily, validated on write, or replaced with a derived value, every method that touches the field directly must be revisited. Self-encapsulation inserts a getter and setter between the field and its consumers within the same class, creating a single point of control for all reads and writes.

This indirection is particularly valuable when subclasses need to participate in the access pattern. A subclass can override the getter to compute the value differently, or override the setter to enforce additional constraints, without the superclass needing to know which implementation is in effect. Self-encapsulation is also a preparatory step for several other refactorings that require controlled access to a field before changing its type or moving it.

## Mechanics

1. Create a getter method that returns the field's value and, if mutation is needed, a setter method that assigns a new value. Both should have at least protected visibility so subclasses can override them.
2. Find every location inside the class where the field is read directly and replace it with a call to the getter. Find every location where the field is written directly and replace it with a call to the setter.
3. Compile and run the test suite after each batch of replacements to confirm that behavior is unchanged. The field itself should remain private; only the accessor methods are exposed.

## Indications

**Signs suggesting this refactoring:**
- A subclass needs to compute a field's value differently from the superclass, but direct field access in the superclass prevents this through overriding.
- A future change will replace the stored field with a computed value, and direct access throughout the class would make that migration expensive.
- The field requires validation or side effects on every write, but some methods bypass the existing setter and assign the field directly.

**When to avoid:**
- The class is final or sealed with no subclasses, the field's representation is stable, and no validation or lazy computation is anticipated. The getter and setter would add indirection with no practical benefit.
- The field is accessed in tight inner loops where even the minimal overhead of a method call is unacceptable, as confirmed by profiling.

## Trade-offs

Self-encapsulation provides a clean extension point for subclasses and a single place to inject validation, logging, or lazy computation. The cost is a small increase in code volume and a layer of indirection that can make the control flow slightly harder to follow at a glance. In classes with few internal consumers of the field, the benefit may be marginal. The technique shines when it is applied as a deliberate stepping stone toward a larger refactoring, such as Replace Type Code with Subclasses, where the getter will become polymorphic. Applied indiscriminately to every field, it generates boilerplate that obscures the class's intent. Reserve it for fields that are likely to change representation or that need to participate in a subclass hook.

## Connections

Self Encapsulate Field is the internal counterpart to Encapsulate Field, which protects a field from external access. It serves as a prerequisite for Replace Type Code with Subclasses and Replace Type Code with State/Strategy, both of which rely on a polymorphic getter to decouple the type code from its consumers. Duplicate Observed Data uses it to intercept field writes and forward them to a domain object. The Template Method pattern benefits from self-encapsulation because the template in the superclass can call the getter, allowing subclasses to supply variant values through overriding. This refactoring also pairs well with Lazy Initialization, where the getter computes the value on first access rather than storing it eagerly.

---

*Based on: Refactoring (Fowler, 1999)*
