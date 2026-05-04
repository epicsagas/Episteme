# Collapse Hierarchy

## Motivation

Collapse Hierarchy is a refactoring technique that merges a superclass and a subclass into a single class when the inheritance relationship no longer carries meaningful behavioral distinction. As a system evolves, features that once justified separate tiers may migrate upward or downward until the two classes become functionally interchangeable. Keeping both alive adds navigation overhead, inflates the mental model developers must hold, and creates a maintenance tax every time a change ripples through a hierarchy that no longer earns its complexity.

The clearest signal is a subclass whose entire public surface could live on the parent without breaking any client, or a superclass that exists only to serve one child. When every method and field on one tier is also present or trivially delegable on the other, the hierarchy has become what developers sometimes call a "phantom level" -- structure without semantics.

## Mechanics

1. Decide which class will survive. Prefer the name that best describes the shared concept; this often means keeping the superclass and discarding the subclass, but the reverse can be appropriate when client code predominantly references the child.
2. If the subclass is being removed, apply Pull Up Field and Pull Up Method to relocate any members that exist only on the child. If the superclass is being removed, apply Push Down Field and Push Down Method instead.
3. Update every type declaration, variable annotation, import statement, and factory call that referenced the discarded class so that it points to the survivor.
4. Run the test suite. Any failure indicates a place where the discarded class carried behavior the surviving class lacks.
5. Delete the empty class definition.

## Indications

**Signs suggesting this refactoring:**
- A subclass overrides no methods and adds no fields beyond what the parent defines.
- Code review reveals that developers treat the two classes interchangeably in discussions and documentation.
- The hierarchy depth exceeds three levels and the middle layer adds no polymorphic behavior.

**When to avoid:**
- Other subclasses still exist beneath the tier being collapsed; merging one branch into its parent can violate the Liskov Substitution Principle for the remaining siblings. For instance, collapsing `Vehicle` into `Car` would force `Airplane` to inherit from `Car`, which is semantically wrong.
- The hierarchy is part of a public API consumed by external code; removing a class is a breaking change that may require a deprecation cycle.

## Trade-offs

Collapsing a hierarchy removes a layer of indirection, which directly reduces the number of files developers must open to trace a feature. It also eliminates the risk that future contributors add code to the wrong tier under a mistaken assumption about where behavior lives. The cost is loss of extensibility: once two tiers merge, re-introducing the split later requires re-establishing the entire hierarchy from scratch. If the system is in active feature growth and new subclasses are on the roadmap, the hierarchy may be premature to collapse even if it currently looks redundant.

## Connections

Collapse Hierarchy is the inverse of Extract Subclass and Extract Superclass -- those introduce hierarchy where none existed, while this removes it. It addresses the Lazy Class smell when the discarded class has become too thin to justify itself, and Speculative Generality when the hierarchy was built for future needs that never materialized. The technique is a specialized form of Inline Class adapted specifically to inheritance relationships. It frequently appears alongside Replace Inheritance with Delegation when a class both collapses and shifts from inheritance-based to composition-based design.

---

*Based on: Refactoring (Fowler, 1999)*
