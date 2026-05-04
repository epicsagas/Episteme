# Pull Up Field

## Motivation

Pull Up Field is a refactoring technique that relocates a data field shared by multiple subclasses into their common superclass, eliminating redundant declarations across the hierarchy. When every child class declares the same field -- often with the same name and type -- the duplication signals that the data concept belongs at a higher level of abstraction. Centralizing the field in the parent makes the hierarchy's shared state visible in one place and prepares the ground for pulling up methods that depend on that field.

The technique is often a preparatory step. Before Pull Up Method can move shared behavior into the superclass, any fields that behavior accesses must already reside there. Pulling up the field first clears the path for consolidating the logic that operates on it.

## Mechanics

1. Verify that the field serves the same semantic purpose in every subclass. Identical names and types are a strong hint, but fields named `id` in one class and `serialNumber` in another might still represent the same concept. Confirm intent, not just form.
2. If field names differ across subclasses, choose a single canonical name and apply Rename Field in each subclass to align them. Update all references within each subclass accordingly.
3. Declare the field in the superclass with the chosen name and type. Use protected or private visibility depending on the language and the access pattern. If the field was private in each subclass and methods from different classes need direct access, consider introducing accessor methods via Self Encapsulate Field.
4. Remove the field declaration from each subclass.
5. Run tests to confirm that all read and write sites still resolve to the correct storage location through inheritance.

## Indications

**Signs suggesting this refactoring:**
- The same field name and type appear in multiple sibling subclasses, and each instance stores conceptually identical data.
- A method being considered for Pull Up Method references a field that does not yet exist in the superclass.
- A new subclass is being written and its field list copies entries from existing siblings.

**When to avoid:**
- The fields share a name but represent different concepts in each subclass -- for example, `rate` meaning tax rate in one class and interest rate in another. Pulling them up implies shared semantics that do not exist.
- Only one subclass currently holds the field, and there is no clear expectation that others will need it. Premature generalization adds clutter to the superclass.

## Trade-offs

A field in the superclass makes the shared data contract explicit: any developer reading the parent immediately understands that all subclasses carry this piece of state. It also reduces the risk of divergence, where one subclass initializes the field differently or adds validation that others lack. The cost is increased coupling. Subclasses can no longer independently change the field's type, name, or visibility without coordinating with the parent. In deeply nested hierarchies, a field pulled up too far -- say, to a root class that dozens of descendants inherit -- can bloat the memory footprint of leaf classes that never use the field. Thoughtful placement at the lowest common ancestor mitigates this risk.

## Connections

Pull Up Field is the inverse of Push Down Field and is typically the first step before Pull Up Method, since shared methods often depend on shared fields. It supports Extract Superclass by populating the new parent with the data it needs. The technique addresses the Duplicate Code smell at the data level and can reveal opportunities for Pull Up Constructor Body when the shared fields all get initialized in the same way across subclasses.

---

*Based on: Refactoring (Fowler, 1999)*
