# Replace Parameter with Explicit Methods

## Motivation

Replace Parameter with Explicit Methods splits a single method that dispatches behavior based on a parameter value into a set of individually named methods, one for each variant. When a method contains a switch or chain of conditionals keyed on an argument, the caller must know which literal value triggers which behavior, and the method name cannot convey that specific behavior because it must be generic enough to cover all cases. A method called `setValue("height", 42)` hides its intent behind a string; `setHeight(42)` states it directly.

The refactoring is most valuable when the dispatch parameter takes a small, stable set of values and each branch contains substantial, independent logic. It turns implicit dispatch into explicit API surface, making each variant discoverable through autocompletion, searchable by name, and individually documentable.

## Mechanics

1. For each distinct value the dispatch parameter can take, create a new method whose name encodes the variant's purpose. Copy the corresponding branch's logic into the new method's body, removing the conditional wrapper.
2. Update every call site that passes a particular dispatch value to invoke the corresponding new method instead. Remove the dispatch argument from the call.
3. Once all call sites have been migrated, delete the original dispatched method. If any callers remain that dynamically select the dispatch value at runtime, consider whether polymorphism via Replace Conditional with Polymorphism would serve them better than explicit methods.

## Indications

**Signs suggesting this refactoring:**
- A method switches on a string, enum, or integer parameter to select among largely independent code paths.
- The dispatch parameter's set of possible values is small and stable, with new values added rarely.
- Callers pass literal constants as the dispatch argument, indicating they already know which variant they want at compile time.
- IDE navigation or autocompletion cannot reveal the available operations because they are hidden behind a single generic method name.

**When to avoid:**
- The dispatch value is determined at runtime and varies per execution, making it impractical to hard-code method calls.
- New variants are added frequently, and each addition would require creating a new public method rather than simply supporting a new parameter value.
- The branches are trivial, such as one-line assignments, and the overhead of separate methods would exceed the clarity gain.

## Trade-offs

Explicit methods make each operation independently discoverable, testable, and documentable. Call sites read as clear imperative statements rather than parameterized dispatch calls, and the compiler can catch misspelled method names where it would silently accept a misspelled string argument. The cost is API surface growth: a class that once exposed one method now exposes several, and each new variant requires a new public method rather than a new constant. When the variant set is large or volatile, this proliferation becomes a maintenance burden that outweighs the clarity benefit. The technique also precludes runtime dispatch entirely; if any caller constructs the dispatch value dynamically, the refactoring cannot serve that caller and a hybrid approach becomes necessary, which adds inconsistency to the API.

## Connections

Replace Parameter with Explicit Methods is the direct inverse of Parameterize Method. It frequently follows Replace Conditional with Polymorphism when the conditional is simple enough that a full subclass hierarchy would be over-engineered. The technique addresses the Switch Statements smell and the Long Method smell by breaking a monolithic dispatcher into focused methods. It pairs well with Rename Method because each new method receives a name that precisely describes its variant's behavior. When the dispatch parameter selects among fundamentally different object behaviors rather than different data, the Strategy pattern or State pattern may be a more appropriate target than explicit methods, providing the same runtime flexibility that this refactoring removes.

---

*Based on: Refactoring (Fowler, 1999)*
