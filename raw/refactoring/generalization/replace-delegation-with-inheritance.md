# Replace Delegation with Inheritance

## Motivation

Replace Delegation with Inheritance is a refactoring technique that converts a composition-based relationship -- where one class holds a reference to another and forwards method calls to it -- into a direct inheritance hierarchy. The need arises when the delegating class ends up exposing the entire public interface of its delegate through thin forwarding methods. Each new method added to the delegate forces a corresponding wrapper in the delegator, creating boilerplate that adds no value. Inheritance eliminates the forwarding layer by letting the delegator become a subclass that inherits every method automatically.

The technique is applicable when the delegating class truly "is-a" specialized form of the delegate -- when every method on the delegate makes semantic sense on the delegator. If the delegator exposes only a subset of the delegate's interface, the "is-a" relationship does not hold and inheritance would pull in unwanted behavior.

## Mechanics

1. Verify that the delegating class forwards calls to every public method of the delegate, or that it would be correct to expose every delegate method through the delegator. If only a subset is forwarded, stop -- this refactoring is inappropriate and would violate the Liskov Substitution Principle.
2. Confirm the delegating class has no existing parent class. Most languages support single inheritance only, so a class already extending another type cannot also extend the delegate.
3. Change the delegating class to extend the delegate class.
4. One by one, remove each forwarding method. Test after each removal to ensure the inherited method works correctly as a replacement.
5. If method names on the delegator differ from those on the delegate, apply Rename Method before removing the forwarding wrapper so that the inherited method carries the correct name.
6. Replace all references to the delegate field with `this` or remove them entirely once forwarding is gone.
7. Delete the delegate field.

## Indications

**Signs suggesting this refactoring:**
- A class contains a field holding an instance of another class and a large set of methods that do nothing but call the same method on that field.
- Every time the delegate class gains a public method, a corresponding wrapper must be added to the delegating class.
- The delegating class adds no behavior beyond forwarding -- its methods contain a single statement that calls through to the delegate.

**When to avoid:**
- The delegating class forwards only a portion of the delegate's interface, meaning the two classes are not in a true "is-a" relationship.
- The delegate might need to be swapped at runtime for a different implementation, a flexibility that inheritance removes.
- The delegating class already extends another class and the language does not support multiple inheritance.

## Trade-offs

Inheritance removes every forwarding method at a stroke, shrinking the class and eliminating the ongoing maintenance cost of keeping wrappers in sync with the delegate's evolving interface. The code becomes easier to read because the class no longer obscures its capabilities behind a layer of indirection. The cost is loss of runtime flexibility: the delegate relationship is now fixed at compile time. If the system later needs to switch implementations dynamically -- for testing, configuration, or plugin loading -- inheritance makes that harder or impossible without another refactoring pass. The technique also couples the subclass to the delegate's entire implementation, including methods that may not be semantically appropriate.

## Connections

Replace Delegation with Inheritance is the inverse of Replace Inheritance with Delegation, which converts a subclass into a composition-based design when inheritance proves inappropriate. It also relates to Remove Middle Man, which eliminates forwarding methods in a delegation chain without necessarily converting to inheritance. The technique addresses the Inappropriate Intimacy smell when that smell stems from a delegating class that is too tightly coupled to its helper object. It should be applied cautiously near the Strategy pattern: where Strategy exploits delegation to swap implementations, replacing it with inheritance forfeits that flexibility.

---

*Based on: Refactoring (Fowler, 1999)*
