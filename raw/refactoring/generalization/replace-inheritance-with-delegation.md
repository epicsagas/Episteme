# Replace Inheritance with Delegation

## Motivation

Replace Inheritance with Delegation is a refactoring technique that converts an inheritance relationship into a composition-based one, where the former subclass holds a reference to the former parent and forwards selected method calls to it. The technique is warranted when a subclass uses only a fraction of its parent's interface, or when the "is-a" relationship implied by inheritance does not hold semantically. A subclass that inherits every method but overrides half of them to throw "unsupported operation" is a strong signal that the hierarchy is lying about the domain.

By replacing inheritance with delegation, the class retains access to the behavior it actually needs while shedding the obligation to support every method the parent defines. This respects the Liskov Substitution Principle by no longer claiming a substitutability that does not exist.

## Mechanics

1. Create a field in the subclass that holds a reference to an instance of the current superclass. Initialize it to `this` temporarily so that existing method calls continue to work during the transition.
2. One by one, change each method in the subclass that calls a superclass method to call through the delegate field instead. Test after each change.
3. For each superclass method that clients call directly on the subclass, create a forwarding wrapper method in the subclass that delegates to the field. Only add wrappers for methods the subclass genuinely needs to expose.
4. Remove the inheritance declaration (`extends`, `:`, or equivalent) so the class no longer inherits from the former parent.
5. Update the delegate field initialization to create a fresh instance of the former superclass rather than referencing `this`.
6. Review all client code. References to superclass-specific methods that are not forwarded will now fail at compile time, which is the desired outcome -- they were the problematic calls the refactoring aims to surface.
7. Run the full test suite.

## Indications

**Signs suggesting this refactoring:**
- A subclass overrides inherited methods to throw exceptions or return no-ops, indicating it cannot fulfill the parent's contract.
- The subclass was created to reuse a few utility methods from the parent rather than to express a true type relationship.
- Code review reveals that the subclass and superclass represent different domain concepts that happen to share some implementation details.

**When to avoid:**
- The subclass genuinely extends the parent's concept and correctly supports every inherited method. In that case, inheritance is the right relationship and converting to delegation adds unnecessary indirection.
- The hierarchy is deep and only one level is problematic; consider introducing an intermediate class instead of dismantling the entire chain.

## Trade-offs

Delegation gives the former subclass precise control over which methods it exposes, eliminating the risk that a client calls an inherited method the class cannot support. It also opens the door to runtime flexibility: the delegate field can hold different implementations, effectively turning the design toward the Strategy pattern. The cost is verbosity. Every forwarded method requires an explicit wrapper, and the total method count in the class may not shrink even though the inheritance relationship is gone. There is also a subtle loss of polymorphic identity: `instanceof` checks and type-based dispatch that relied on the inheritance chain no longer work unless the class explicitly implements the parent's interface.

## Connections

Replace Inheritance with Delegation is the inverse of Replace Delegation with Inheritance. It directly addresses the Refused Bequest smell by removing inherited methods that a class does not want, and the Inappropriate Intimacy smell when a subclass reaches deeply into parent internals. The technique naturally leads to the Strategy pattern, since the delegate field can hold any implementation conforming to the expected interface. It also relates to Extract Interface: after breaking the inheritance link, extracting an interface from the former parent lets the delegating class declare its dependency as an abstraction rather than a concrete type, improving testability and decoupling.

---

*Based on: Refactoring (Fowler, 1999)*
