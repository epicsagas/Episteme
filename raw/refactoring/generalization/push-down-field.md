# Push Down Field

## Motivation

Push Down Field is a refactoring technique that relocates a field from a superclass into the specific subclasses that actually use it. The field may have been placed in the parent under the assumption that all children would need it, or it may have become unused by most subclasses after a feature was removed or restructured. In either case, keeping the field in the superclass forces every child to carry data it does not need, which inflates object memory and misleads developers into thinking the field plays a role in the base concept.

The technique is often paired with Extract Subclass. As specialized behavior is pushed down into a new child class, the data that supports that behavior should follow, keeping the parent class lean and focused on truly universal state.

## Mechanics

1. Identify every subclass that reads or writes the field. A field used by only one or two children out of a larger set is a strong candidate for relocation.
2. For each subclass that uses the field, declare a private or protected field with the same name and type. If different subclasses need different types or initialization logic, allow the field declarations to diverge accordingly -- that independence is part of the benefit.
3. Remove the field declaration from the superclass.
4. If any superclass method references the field, either pull that method down as well using Push Down Method, or introduce abstract accessor methods that subclasses implement to provide the field's value.
5. Run tests to verify that each subclass still functions correctly with its own copy of the field.

## Indications

**Signs suggesting this refactoring:**
- A superclass field is accessed by only a subset of subclasses, while others never reference it.
- The field's value is always a sentinel or default for certain subclasses, indicating it carries no meaningful data for those children.
- Object diagrams show that instances of some subclasses waste memory on fields that are never populated.

**When to avoid:**
- The field is used by all or nearly all subclasses; pushing it down would duplicate the declaration without reducing coupling.
- The field participates in shared logic within a superclass method, and extracting that method to each subclass would fragment behavior that is genuinely universal.

## Trade-offs

Relocating a field to the subclasses that need it makes the superclass smaller and more honest about the data it truly represents. Each subclass becomes self-contained regarding the field, which allows independent evolution -- one child can change the field's type or validation without affecting siblings. The cost is potential duplication if multiple subclasses declare the same field independently, and a loss of the explicit signal that the data concept is shared. Developers must recognize the conceptual connection themselves rather than seeing it unified in the parent. This trade-off favors the refactoring when the subclasses genuinely need different implementations and disfavors it when the field is truly common but merely underused today.

## Connections

Push Down Field is the inverse of Pull Up Field and frequently accompanies Push Down Method and Extract Subclass in a refactoring sequence. It addresses the Refused Bequest smell, where a subclass inherits data it never uses. When a superclass accumulates multiple fields that only one child uses, the combination of Push Down Field and Extract Subclass can split the hierarchy into a clean base and specialized children. The technique also relates to Replace Inheritance with Delegation when the "refused" fields suggest that inheritance itself is the wrong relationship for the affected subclass.

---

*Based on: Refactoring (Fowler, 1999)*
