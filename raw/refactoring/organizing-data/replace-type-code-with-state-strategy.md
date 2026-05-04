# Replace Type Code with State/Strategy

## Motivation

A type code that controls class behavior through conditional logic creates a maintenance burden: every new variant requires modifying the existing class to add another branch, violating the Open/Closed Principle. Subclassing would distribute the behavior across a hierarchy, but it is impractical when the type code can change during an object's lifetime or when the host class already participates in a different inheritance chain. The State and Strategy patterns offer a compositional alternative: the host class delegates behavior to a separate object that can be swapped at runtime, and each variant becomes a distinct class implementing a common interface.

The distinction between the two patterns is pragmatic. Strategy applies when the type code selects between alternative algorithms for a single operation. State applies when the type code pervasively affects the object's behavior across multiple operations, effectively modeling a lifecycle or mode. Both patterns replace conditionals with polymorphic dispatch to a companion object.

## Mechanics

1. Apply Self Encapsulate Field to the type code field so that all access flows through a getter. This establishes the hook point where the delegation will be inserted.
2. Define an abstract class or interface representing the state or strategy role. Declare the methods that currently contain type-code conditionals as abstract methods on this interface.
3. Create a concrete subclass for each type code value, implementing the abstract methods with the behavior appropriate to that variant. Include a static factory method on the interface that maps a type code value to the corresponding subclass instance.
4. Change the host class's field type from the primitive code to the new interface. Modify the setter to use the factory method to select the correct implementation, and update the getter to return the interface type.
5. Progressively move fields and methods that depend on the type code from the host class into the appropriate state or strategy subclass using Push Down Method and Push Down Field.
6. Replace each conditional on the type code with a polymorphic call to the delegated object. Once all conditionals are eliminated, the primitive type code field can be removed entirely.

## Indications

**Signs suggesting this refactoring:**
- A type code field drives multiple conditional branches scattered across the class, and new variants are expected.
- The type code can change during an object's lifetime, ruling out Replace Type Code with Subclasses which requires the code to be fixed at construction.
- The host class already extends another class, making further subclassing impossible in single-inheritance languages.

**When to avoid:**
- The type code has only two or three values, the conditionals are simple, and no new variants are anticipated. The overhead of separate classes outweighs the clarity gained.
- The type code carries no behavioral implications and is used purely for classification. Replace Type Code with Class is sufficient in that case.
- The behavior differences are so minor that moving them into separate classes would scatter trivial amounts of logic across many files, reducing rather than improving comprehension.

## Trade-offs

Delegating to State or Strategy objects enables runtime behavior changes and aligns with the Open/Closed Principle, but it introduces an additional layer of indirection. Every operation that formerly checked a field now makes a virtual method call on a delegate, which can complicate stack traces and make control flow harder to follow during debugging. The number of classes increases: one interface plus one concrete class per variant. In systems where the variant set is small and stable, this proliferation can feel excessive. On the other hand, when variants are numerous or growing, the pattern prevents the host class from becoming a monolithic switchboard. The key trade-off is between the upfront complexity of the pattern infrastructure and the long-term cost of maintaining and extending conditional logic.

## Connections

This refactoring is one of three approaches to eliminating type codes. Replace Type Code with Class handles the data-only case, Replace Type Code with Subclasses handles the behavioral case when the code is immutable, and this refactoring handles the behavioral case when the code can change. It implements the State pattern for lifecycle modeling or the Strategy pattern for algorithm selection. Self Encapsulate Field is a prerequisite. Replace Conditional with Polymorphism is the follow-up that completes the elimination of switch statements. The resulting design often resembles the Command pattern when individual strategy objects encapsulate discrete operations.

---

*Based on: Refactoring (Fowler, 1999)*
