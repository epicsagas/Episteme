# Replace Subclass with Fields

## Motivation

A class hierarchy exists where each subclass does nothing more than return a different constant value through an overridden method. The subclasses carry no unique state, no distinct behavior beyond the constant, and no reason to exist as separate types. This over-engineering often arises after larger refactoring efforts have moved substantial logic elsewhere, leaving behind a hollow taxonomy. Maintaining a subclass for each variant forces developers to create a new class every time a new value is needed, even though a simple field assignment would suffice.

Collapsing the hierarchy into a single class that stores the varying values as fields eliminates the structural overhead. New variants become a constructor argument or a factory call rather than a new source file, and the codebase shrinks by the number of removed subclasses.

## Mechanics

1. Apply Replace Constructor with Factory Method to the superclass. This gives the factory method control over which subclass to instantiate, and later lets it return the superclass directly once subclasses are gone.
2. Replace every direct subclass constructor call with an invocation of the factory method, passing any distinguishing values as parameters.
3. Add fields to the superclass to hold the values that were previously returned by subclass methods. Create a protected or private constructor on the superclass that accepts these values and initializes the fields.
4. Modify each subclass constructor to call the new superclass constructor, passing its specific constant as an argument. Then move the constant-returning methods into the superclass, implementing them to return the corresponding field value.
5. Apply Inline Method to absorb any remaining subclass-specific constructor logic into the factory method. Once a subclass has no unique behavior or state, delete it.
6. Repeat for each subclass until only the superclass remains. The factory method now returns the superclass type directly, configured with the appropriate field values.

## Indications

**Signs suggesting this refactoring:**
- Every subclass in a hierarchy exists solely to override one or two methods that return fixed constants, such as a label, a color code, or a rate multiplier.
- Adding a new variant requires creating an entire subclass file, even though the new variant differs from existing ones only in a few literal values.
- Previous refactoring rounds have already stripped the subclasses of all meaningful behavior, leaving them as trivial stubs.

**When to avoid:**
- Subclasses carry genuine behavioral differences beyond constant values, such as distinct algorithms or state transitions.
- The hierarchy is expected to grow in complexity, and subclasses will soon accumulate unique methods or fields that justify their existence.
- The application uses runtime type checks or visitor dispatch based on subclass identity, and collapsing the hierarchy would break those mechanisms.

## Trade-offs

Removing subclasses simplifies the class structure and makes new variants cheap to add, but it discards the polymorphic dispatch mechanism. If a future requirement demands behavior that varies per variant, the superclass must absorb conditional logic or the hierarchy must be reintroduced. Fields also lose the type-safety that distinct subclasses provide: a caller cannot pattern-match on the variant type, and the compiler cannot enforce that only valid combinations of field values exist. For small, stable sets of variants this is an acceptable trade-off, but for growing or behavior-rich taxonomies the subclass hierarchy remains the stronger design.

## Connections

The inverse transformation is Replace Type Code with Subclasses, which creates subclasses to handle behavioral variation. This refactoring often follows Replace Conditional with Polymorphism if the polymorphism turned out to be trivial. Replace Constructor with Factory Method is a prerequisite step that enables the migration. The Speculative Generality smell is a common trigger, indicating that the hierarchy was built in anticipation of variation that never materialized. The resulting flat class may benefit from Replace Type Code with Class if the retained field still carries a coded value.

---

*Based on: Refactoring (Fowler, 1999)*
