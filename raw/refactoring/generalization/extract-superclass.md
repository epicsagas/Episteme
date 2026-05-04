# Extract Superclass

## Motivation

Extract Superclass is a refactoring technique that creates a shared parent class to hold common fields and methods discovered in two or more sibling classes. Duplication across classes is one of the most reliable indicators that a generalization opportunity exists: when `Employee` and `Contractor` both carry `name`, `address`, and a `printLabel` method, those concerns belong in a single place rather than scattered across parallel implementations.

The technique is retroactive by nature. Developers often build classes independently before realizing they share a conceptual core. Rather than tolerating duplicate maintenance -- fixing a bug in the label-printing logic twice, for instance -- this refactoring lifts the shared core into a superclass that both classes inherit from.

## Mechanics

1. Create an abstract class with a descriptive name that captures the shared concept. Avoid leaking implementation details into the name; prefer `Party` or `Person` over `EmployeeContractorBase`.
2. Identify shared fields and move them upward with Pull Up Field. Prioritize fields that shared methods depend on, since those relocations will be prerequisite for the next step.
3. Identify shared methods and move them upward with Pull Up Method. If a method references a field that has not yet been pulled up, move that field first.
4. If constructors across the subclasses share initialization logic, apply Pull Up Constructor Body to consolidate the common setup into the new superclass constructor.
5. Update client code to reference the superclass type wherever only the shared capabilities are needed. This widens the polymorphic surface and reduces coupling to concrete classes.
6. Delete the now-redundant code from each subclass.

## Indications

**Signs suggesting this refactoring:**
- Two or more classes contain fields with identical names and types, or fields with different names that serve the same purpose.
- Methods with the same signature and body appear in multiple classes, indicating copy-paste duplication rather than coincidence.
- A concept in the domain model -- such as "party" or "account" -- is implicit in several classes but never formalized as a type.

**When to avoid:**
- The classes already have distinct superclasses and the language does not support multiple inheritance. In that case, Extract Interface can capture the shared contract without requiring a common parent.
- The shared behavior is superficial and the classes are conceptually unrelated. Forcing a superclass onto classes that happen to share a utility method introduces a misleading "is-a" relationship where composition would be more honest.

## Trade-offs

Consolidating duplicated fields and methods into a superclass yields a single source of truth, which directly reduces the surface area for bugs and the effort required for future changes. The trade-off is coupling: subclasses are now bound to their parent's implementation details. A change to a pulled-up method affects all children, which is beneficial when the behavior should be uniform but dangerous when subclasses need to diverge later. The technique also adds a layer to the inheritance hierarchy, slightly increasing navigation effort during debugging. In codebases where inheritance depth is already a concern, Extract Class or Extract Interface may provide the deduplication benefit without deepening the tree.

## Connections

Extract Superclass is the inverse of Collapse Hierarchy, which removes a parent class that no longer justifies itself. It relies on Pull Up Field, Pull Up Method, and Pull Up Constructor Body as sub-operations. Where Extract Superclass captures both contract and shared code, Extract Interface captures only the contract, making the two complementary techniques. The refactoring eliminates Duplicate Code across class boundaries and addresses the Parallel Inheritance Hierarchies smell when the new parent unifies what were previously parallel trees.

---

*Based on: Refactoring (Fowler, 1999)*
