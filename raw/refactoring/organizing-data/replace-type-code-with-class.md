# Replace Type Code with Class

## Motivation

Type codes are encoded values, often integers or strings, used to distinguish between categories of an entity. A customer might carry a type code of 1 for regular, 2 for preferred, and 3 for VIP, or a shipment might use strings like "AIR", "GROUND", and "SEA". These codes are fragile because any integer or string is accepted by the compiler, invalid assignments go undetected until runtime, and the meaning of each code is scattered across comments and documentation rather than embedded in the type system.

Wrapping the type code in a dedicated class gives each valid value its own static instance, turning arbitrary primitive assignments into type-checked references. The compiler now rejects invalid codes, IDEs can suggest valid options, and the definition lives in one authoritative place. This refactoring is the lightest-weight cure for type codes, appropriate when the codes carry no behavioral implications.

## Mechanics

1. Create a new class to represent the type code. Give it a private constructor that accepts the underlying primitive value and stores it in a final field. Provide a public getter for the stored value.
2. Inside the new class, declare a static final instance for each valid type code value. Each instance is constructed with its specific primitive value and given a descriptive name, such as `CustomerType.REGULAR`.
3. Change the field type in the original class from the primitive to the new class. Update any constructors and setters to accept and store instances of the new class rather than raw primitives.
4. Replace every literal type code value throughout the codebase with a reference to the corresponding static instance. Remove any now-obsolete constant definitions that held the raw codes.

## Indications

**Signs suggesting this refactoring:**
- A field accepts a narrow set of primitive values, and invalid values are caught only at runtime, perhaps through a crash or a silently incorrect calculation.
- Constants named with ALL_CAPS naming conventions are used to label valid type codes, but nothing prevents a caller from passing an arbitrary integer or string.
- The type code is used purely for data classification, never to drive conditional behavior such as branching or algorithm selection.

**When to avoid:**
- The type code controls program flow through conditional statements. In that case Replace Type Code with Subclasses or Replace Type Code with State/Strategy is more appropriate because they address the behavioral dimension as well.
- The set of valid codes is extremely large or dynamic and cannot be enumerated as static instances. A lookup table or a configuration-driven approach may be better suited.

## Trade-offs

A type-safe class eliminates an entire class of bugs by constraining assignments to valid instances, but it adds a new class and changes every call site that constructs or compares type codes. For small, stable codebases the migration cost is modest; for large systems with hundreds of call sites it can be a significant undertaking. The refactoring does not address behavioral variation: if the type code currently drives switch statements, those conditionals remain. Its strength is strictly in the data dimension, making invalid states unrepresentable at the type level. When behavioral polymorphism is also needed, this refactoring serves as a stepping stone toward Replace Type Code with Subclasses or Replace Type Code with State/Strategy.

## Connections

This refactoring addresses the Primitive Obsession smell by replacing a raw primitive with a meaningful type. It is the simplest of the three type-code refactorings; Replace Type Code with Subclasses adds behavioral polymorphism via inheritance, and Replace Type Code with State/Strategy adds it via composition. The resulting class resembles a typesafe enumeration, which many modern languages support natively through enum constructs. Self Encapsulate Field may be useful as a preparatory step to control access to the type code field before changing its type.

---

*Based on: Refactoring (Fowler, 1999)*
