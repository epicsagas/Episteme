# Replace Magic Number with Symbolic Constant

## Motivation

A magic number is a numeric literal embedded directly in code without an explanatory name. Its purpose is opaque to anyone reading the code, and changing it requires hunting through the entire codebase for every occurrence, hoping not to miss one or accidentally change a different number that happens to share the same value. This fragility is amplified when the same number appears in multiple locations with subtly different meanings, such as the number 60 meaning both seconds-per-minute and the maximum widget count in different contexts.

Replacing a magic number with a named constant makes the intent explicit and centralizes the value's definition. A single named constant can be updated in one place, and its name serves as inline documentation that never drifts out of sync with the value it holds.

## Mechanics

1. Identify a numeric literal whose purpose is not immediately obvious from context. Verify that the same value does not serve a different purpose elsewhere, because sharing a constant between unrelated concepts creates a false coupling.
2. Declare a constant with a descriptive name in an appropriate scope, such as a class-level static field or a module-level constant, and assign it the literal value.
3. Replace every occurrence of the literal that shares the same meaning with a reference to the named constant. For each replacement, confirm that the number's purpose aligns with the constant's name before substituting.
4. If the constant is used across multiple classes, consider consolidating it in a shared constants class or an enumeration, but avoid creating a catch-all constants file that becomes a dumping ground for unrelated values.

## Indications

**Signs suggesting this refactoring:**
- A numeric literal appears in business logic without a comment or a name explaining what it represents, forcing readers to infer its meaning from surrounding code.
- The same numeric value appears in multiple locations, and updating it requires a manual search-and-replace that risks missing an occurrence or accidentally changing an unrelated instance.
- A formula contains a constant whose units are ambiguous, such as multiplying by 9.81 without indicating that it is the gravitational acceleration in meters per second squared.

**When to avoid:**
- The number's purpose is self-evident from context, such as the zero in a loop initialization or the one in an increment expression. Naming these adds verbosity without clarity.
- The number is a well-known mathematical constant already understood by domain practitioners, though even pi and e benefit from named constants if the codebase uses them in non-obvious ways.
- The number functions as a type code controlling program behavior. In that case, Replace Type Code with Class or Replace Type Code with Subclasses is the more appropriate refactoring.

## Trade-offs

Named constants improve readability and maintainability, but they can proliferate if applied indiscriminately. A constants file that grows without bound becomes its own maintenance burden, and constants defined far from their usage can be harder to discover than a well-placed inline comment. The discipline lies in replacing numbers whose meaning is non-obvious or whose value may change, while leaving truly self-explanatory literals alone. Enumerations offer a middle ground for sets of related constants, providing both names and a scope that prevents accidental misuse, but they are heavier than a simple constant declaration and may not suit solitary values.

## Connections

This refactoring directly addresses the Magic Number smell and is a lightweight form of the broader Primitive Obsession remedy. When a magic number functions as a type code, Replace Type Code with Class, Replace Type Code with Subclasses, or Replace Type Code with State/Strategy provides a more robust solution. Replace Data Value with Object generalizes this idea by wrapping not just a number but any primitive in a dedicated class. Extract Constant is the language-agnostic name for the mechanical step of creating the named constant.

---

*Based on: Refactoring (Fowler, 1999)*
