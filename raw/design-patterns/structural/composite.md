# Composite

## Essence
Composite is a structural design pattern that lets clients treat individual objects and groups of objects uniformly through a shared interface, enabling recursive tree structures where every node responds to the same operations. Leaf nodes perform the operation directly, while composite nodes delegate to their children and optionally aggregate the results. This eliminates type-checking and branching in client code because every element, whether simple or nested, exposes the same contract.

## Motivation
A project management tool models tasks as a hierarchy: a top-level epic contains stories, each story contains subtasks, and subtasks may themselves be broken into finer steps. The UI must render the full tree, compute the total estimated hours, and mark a branch as complete when all its descendants are done. Without a uniform interface, the rendering code must check whether each node is a leaf or a container and branch accordingly, duplicating aggregation logic at every level. Adding a new node type (for instance, a milestone that groups stories by date) forces changes in every function that traverses the tree.

Composite solves this by defining a `TaskComponent` interface with methods like `estimatedHours()`, `markComplete()`, and `render()`. A `Task` leaf implements these methods directly, returning its own hours and marking itself done. A `TaskGroup` composite stores a list of child `TaskComponent` objects and implements the same methods by iterating over children and summing their hours or delegating the operation downward. The UI code holds a reference to the root `TaskComponent` and calls its methods without ever checking whether it is a leaf or a group. Adding a `Milestone` type requires only a new class implementing the interface, with no changes to traversal or rendering code.

## Participants
The component interface declares the operations that both leaves and composites must support. Leaf classes implement the interface by performing the operation directly, with no children to manage. Composite classes also implement the interface but additionally store a collection of child components, delegating operations to each child and often combining their results. Composites also provide methods to add and remove children. The client works exclusively through the component interface, treating every node identically regardless of its internal complexity. This uniformity means a client can pass a subtree or a single leaf to the same function and get correct behavior in both cases.

## Application

**Use when:**
- The domain naturally forms a tree or recursive hierarchy (file systems, organization charts, scene graphs, nested menus)
- Client code should process individual elements and groups of elements through the same operations without conditional type checks
- New element types will be added frequently, and the system must accommodate them without modifying traversal or processing logic

**Prefer alternatives when:**
- The hierarchy is flat and will never be nested (a simple list suffices)
- Leaf and composite operations are fundamentally different and forcing a shared interface creates awkward default implementations in leaf nodes
- Strict type safety is paramount and treating everything as a component would obscure important distinctions (the Visitor pattern can provide type-safe traversal instead)

## Consequences
Composite delivers the striking benefit of treating complex structures as simply as individual objects, which dramatically simplifies client code and makes recursive algorithms natural to express. New node types slot in without touching existing code, supporting the Open/Closed Principle. However, the shared interface can become a liability when leaves and composites need genuinely different operations: forcing both to implement a method that only makes sense for one side leads to empty or throwing implementations, which weakens the type contract. Designers must decide whether the component interface should include child-management methods (which leaves must reject) or restrict those to the composite class (which forces clients to perform type checks in certain scenarios). Despite this tension, Composite remains the go-to pattern for any tree-shaped domain model.

## Relations
Composite and Decorator share a recursive structure, but Decorator adds responsibilities to a single object by wrapping it, while Composite aggregates many objects under a shared interface. Builder constructs Composite trees naturally, calling child-creation steps recursively to assemble the hierarchy. Visitor complements Composite by encapsulating operations that would otherwise live inside each component, especially useful when the component hierarchy is stable but operations change frequently. Iterator can traverse a Composite tree without exposing its internal structure. Flyweight can optimize memory usage when a Composite contains many identical leaf objects by sharing intrinsic state across them. Chain of Responsibility is sometimes combined with Composite so that requests propagate up or down the tree until a handler processes them.

---

*Based on: Design Patterns (Gamma, Helm, Johnson, Vlissides, 1994)*
