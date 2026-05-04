# Form Template Method

## Motivation

Form Template Method is a refactoring technique that captures a shared algorithmic skeleton in a superclass while delegating individual steps to subclass implementations. The telltale sign is a set of subclasses whose methods follow the same high-level sequence of operations but differ in the details of one or two steps. Without this refactoring, each subclass duplicates the overall structure, forcing developers to maintain identical control flow in multiple places and increasing the risk that a structural change to the algorithm is applied inconsistently.

The technique implements the Template Method design pattern by giving the parent class a concrete method that calls a series of abstract or hook methods. Subclasses then override only the steps that vary. This arrangement centralizes the algorithm's invariant structure while preserving flexibility at the points of variation.

## Mechanics

1. Examine the methods in each subclass that perform the similar algorithm. Use Extract Method liberally to decompose each one into small, named steps. The goal is to reach a state where each step is a single method call whose intent is clear from its name.
2. Identify which steps are identical across all subclasses. Apply Pull Up Method to move those shared implementations to the superclass.
3. For steps that differ between subclasses, apply Rename Method so every subclass uses the same method name for the analogous step. The signatures must match exactly.
4. Declare the varying-step methods as abstract in the superclass (or provide default hook implementations if some subclasses can share a fallback).
5. Create or promote the top-level method in the superclass -- the template method -- that calls the shared steps and the abstract steps in the correct order. Delete the original algorithm methods from each subclass.
6. Verify that all call sites invoke the template method through the superclass type.

## Indications

**Signs suggesting this refactoring:**
- Multiple subclasses contain methods with different names but identical control flow, differing only in specific operations within the flow.
- A comment or code block like `// step 1, step 2, step 3` appears in several places with the same ordering but different step details.
- Fixing a structural bug in the algorithm requires editing the same boilerplate in every subclass.

**When to avoid:**
- The algorithms share fewer than half their steps; the template method will end up with so many abstract hooks that reading the parent class tells the developer nothing about what actually happens.
- The subclass methods are still evolving rapidly and their step sequences have not stabilized. Premature templating locks in a structure that may need to change, making future refactoring harder rather than easier.

## Trade-offs

The central benefit is a single location for the algorithm's structure. When the invariant steps change, one edit propagates correctly to every subclass. The Open/Closed Principle is also served: adding a new variant means creating a new subclass that fills in the abstract steps, with no modification to existing code. The downside is indirection. A developer reading a subclass no longer sees the full algorithm in one place; they must jump to the parent to understand the ordering and then back to the child for the specific step implementations. This back-and-forth is manageable with two or three steps but becomes burdensome when the template method orchestrates many hooks. Over-templating can also lead to a proliferation of tiny single-method subclasses that add hierarchy depth without proportional clarity.

## Connections

Form Template Method is a direct implementation of the Template Method design pattern and relies on Pull Up Method as its primary sub-operation. It frequently follows Extract Method, since decomposing the original algorithm into discrete steps is a prerequisite. The technique addresses Duplicate Code at the algorithmic level -- a coarser grain than Extract Method or Pull Up Method address individually. It also relates to the Strategy pattern: where Template Method uses inheritance to vary steps, Strategy uses composition. When the inheritance-based approach begins to feel rigid, Replace Inheritance with Delegation can convert the template hierarchy into a strategy-based design.

---

*Based on: Refactoring (Fowler, 1999)*
