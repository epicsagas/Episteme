# Replace Conditional with Polymorphism

## Motivation

Replace Conditional with Polymorphism transforms type-checking conditionals, such as `switch` or `if-else` chains that inspect an object's type code or category, into polymorphic method calls distributed across a class hierarchy. When multiple methods in the same class contain parallel conditional structures that select behavior based on the same discriminator, the conditionals become a maintenance liability: every new variant requires finding and updating every switch statement. Polymorphism inverts this relationship. Instead of client code asking an object what it is and then deciding what to do, each variant class simply implements the operation differently. Adding a new variant requires only a new subclass, with no changes to existing callers. This inversion is the practical realization of the Tell-Don't-Ask principle and directly supports the Open/Closed Principle.

## Mechanics

1. Ensure the conditional logic is isolated in its own method. If the switch is interleaved with unrelated code, apply Extract Method first.
2. Establish a class hierarchy if one does not already exist. Use Replace Type Code with Subclasses when each variant is a permanent classification, or Replace Type Code with State/Strategy when the variant can change at runtime.
3. For the first branch of the conditional, create an override of the method in the corresponding subclass. Copy the branch's body into the override, adapting any references that differ in the subclass context.
4. Delete the processed branch from the original conditional in the base class, then compile and test to confirm the subclass produces the correct behavior.
5. Repeat for each remaining branch, moving one branch at a time into its corresponding subclass override. After every migration, run tests to catch regressions early.
6. Once all branches have been relocated, delete the now-empty conditional from the base class. Declare the method as abstract, or provide a default implementation if a catch-all behavior is needed.
7. Review all other methods in the base class for parallel conditional structures that use the same type code. Apply the same migration to each, ensuring every conditional that depends on the discriminator is eliminated.

## Indications

**Signs suggesting this refactoring:**
- A `switch` or cascading `if-else` that inspects a type code, enum value, or string tag to select behavior
- Multiple methods in the same class containing parallel conditional structures keyed on the same discriminator
- A comment like "add new type here" appearing in several places, signaling that the conditional is a change hotspot
- Frequent bugs caused by adding a new variant in some switch statements but forgetting others

**When to avoid:**
- The conditional has only two branches and is unlikely to grow, making a class hierarchy unnecessary overhead
- The type discriminator can change at runtime but the language or design does not support hot-swapping the Strategy object
- The conditional logic is trivial, such as returning a constant per type, and a simple lookup table would suffice
- Introducing a class hierarchy would violate the project's architectural constraints, such as a strict data-only domain model

## Trade-offs

Polymorphism eliminates conditional duplication and makes the system extensible by design: new variants are added as new classes without modifying existing code. The Open/Closed Principle is satisfied, and the Tell-Don't-Ask principle is honored. However, the refactoring introduces structural complexity: a class hierarchy with multiple subclasses, each carrying a small method body, can be harder to navigate than a single switch statement, especially when the total logic is small. Dispatch becomes implicit, so a developer reading the base class cannot see all the behavior in one place; they must trace through the subclass hierarchy. Serialization, persistence, and framework integration can also become more complex when objects are identified by their concrete type rather than a simple code. The refactoring yields the greatest return when the conditional is replicated across multiple methods and when new variants are added regularly.

## Connections

This refactoring directly eliminates the Switch Statements smell and is one of the most powerful tools for achieving adherence to the Open/Closed Principle. It depends on prior refactorings such as Extract Method to isolate the conditional logic and Replace Type Code with Subclasses or Replace Type Code with State/Strategy to establish the class hierarchy. The Strategy pattern and the State pattern are the structural foundations that make this refactoring possible. Introduce Null Object is a specialized application where one of the variants represents the absence of a real object. Decompose Conditional can serve as a preparatory step by making each branch's logic explicit before moving it into a subclass.

---

*Based on: Refactoring (Fowler, 1999)*
