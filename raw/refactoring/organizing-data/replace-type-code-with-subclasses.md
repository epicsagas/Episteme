# Replace Type Code with Subclasses

## Motivation

When a type code field governs behavior through conditional branches, the class accumulates switch statements that grow with every new variant. Each branch encodes knowledge about what makes one variant different from another, but that knowledge lives inside procedural conditionals rather than being distributed across the type system. Subclassing replaces these conditionals with polymorphic dispatch: each variant becomes its own class, and the behavior that was once guarded by an if-else chain becomes the natural implementation of an overridden method.

This refactoring is appropriate when the type code is immutable, meaning it is set at construction and never changes. If the code can vary during the object's lifetime, composition-based approaches such as Replace Type Code with State/Strategy are required instead, because an object cannot change its class at runtime in most languages.

## Mechanics

1. Apply Self Encapsulate Field to the type code field, introducing a getter that mediates all access. This getter will later become polymorphic, with each subclass returning its own fixed value.
2. Create a distinct subclass for each type code value. Override the getter in each subclass to return the constant corresponding to that variant.
3. Make the superclass constructor private or protected and introduce a static factory method that accepts the type code and returns an instance of the appropriate subclass. This centralizes the instantiation decision and prevents callers from depending on concrete subclasses directly.
4. Remove the type code field from the superclass and declare the getter as abstract, forcing each subclass to provide the value. This step ensures that no variant can be instantiated without specifying its identity.
5. Identify fields and methods that apply to only some variants and move them into the corresponding subclasses using Push Down Field and Push Down Method. This progressively tailors each subclass to its specific responsibility.
6. Apply Replace Conditional with Polymorphism to each conditional that tests the type code, replacing it with overridden methods in the subclasses.

## Indications

**Signs suggesting this refactoring:**
- A field holds a coded value that drives switch or if-else chains throughout the class, and the set of possible values is known and stable.
- The type code is assigned once at construction and never modified, making a permanent subclass identity a natural fit.
- New variants are anticipated, and the current conditional structure is becoming unwieldy to extend.

**When to avoid:**
- The type code can change after construction. Subclass identity is fixed at instantiation in most object-oriented languages, so mutable type codes require the State or Strategy pattern instead.
- The host class already has subclasses for a different reason, creating a conflicting inheritance hierarchy. Composition-based alternatives avoid this collision.
- The number of variants is very small, such as two, and the conditionals are trivial. The overhead of multiple classes is not justified.

## Trade-offs

Subclassing distributes variant-specific behavior into focused classes, improving adherence to the Single Responsibility Principle and making new variants a matter of adding a class rather than editing switch statements. However, it locks each object into a single variant for its entire lifetime, which is limiting if the domain requires dynamic reclassification. The inheritance hierarchy also introduces a permanent structural commitment: every variant is a compile-time type, which can complicate serialization, persistence, and ORM mapping. In domains where variant-specific state is minimal and the behavior differences are confined to a few methods, the overhead of a full class hierarchy may outweigh the clarity gains. Weigh the number and complexity of conditionals against the cost of the new classes.

## Connections

This refactoring occupies the middle ground of the three type-code elimination strategies: Replace Type Code with Class handles data-only codes, and Replace Type Code with State/Strategy handles mutable behavioral codes. Self Encapsulate Field is a prerequisite step. Replace Conditional with Polymorphism is the natural continuation. The resulting hierarchy may eventually be simplified by Replace Subclass with Fields if behavioral differences turn out to be trivial. This refactoring addresses the Primitive Obsession smell and frequently co-occurs with the Switch Statements smell. It implements the Template Method pattern implicitly when the superclass defines an algorithm skeleton that subclasses customize through overridden methods.

---

*Based on: Refactoring (Fowler, 1999)*
