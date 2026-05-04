# Visitor

## Essence
Visitor is a behavioral design pattern that separates operations from the object structures they act upon, enabling new behaviors to be added without modifying the element classes. A visitor object implements a set of methods, one for each element type in the structure, and each element accepts a visitor and dispatches to the method matching its own type through a double-dispatch mechanism. This lets you define families of related operations in isolated visitor classes rather than sprinkling behavior across every element.

## Motivation
Picture a compiler's abstract syntax tree containing nodes for assignments, function calls, binary expressions, and literals. The engineering team needs to generate x86 assembly, emit LLVM IR, run a type checker, perform constant folding, and produce source-mapped debug info, with more passes planned. Adding a method for each pass to every AST node class would bloat the node hierarchy and force a recompile of the entire AST module whenever a new analysis pass is introduced.

Visitor solves this by keeping node classes stable with a single accept method. Each compiler pass becomes a visitor class with a visit method specialized for each node type. When a node accepts a visitor, it calls back into the visitor with itself as an argument, and the visitor's overloaded method for that specific node type executes. New passes are added by writing new visitor classes, leaving the AST untouched.

## Participants
The Visitor interface declares a visit method for each concrete element type in the object structure. Concrete Visitor classes implement these methods to define a coherent operation across all element types. The Element interface declares an accept method that takes a visitor argument. Concrete Elements implement accept by calling the visitor's method corresponding to their own type, enabling double dispatch. The Object Structure, often a composite collection, enumerates its elements and passes the visitor to each one's accept method.

## Application

**Use when:**
- you need to perform distinct, unrelated operations across a stable set of element classes
- the element class hierarchy changes infrequently but new operations are added often
- an operation must behave differently depending on the concrete runtime type of elements, and type-testing conditionals would be cumbersome

**Prefer alternatives when:**
- the element class hierarchy is volatile, because adding a new element type forces changes to every existing visitor
- operations are simple and few, making the accept-visit protocol unnecessary overhead
- the language supports pattern matching or multimethods that handle type-based dispatch natively

## Consequences
Visitor earns strong Open/Closed Principle compliance for operations: new behaviors arrive as new visitor classes without touching element code. It also supports Single Responsibility by grouping related operations in one visitor instead of scattering them across elements. Visitor objects can accumulate state as they traverse a structure, making aggregation and collection operations natural. The principal drawback is the element hierarchy must remain stable; adding a new element type requires updating every visitor interface and all concrete visitors. Visitor methods may also need access to private element fields, forcing element classes to expose internal state through accessors that weaken encapsulation.

## Relations
Visitor is often described as a powerful generalization of Command: while Command encapsulates a single operation on a single receiver, Visitor encapsulates a family of operations across a family of receivers. Composite provides the object structures that Visitor traverses, making them a common pairing. Iterator handles uniform traversal of homogeneous collections, whereas Visitor handles heterogeneous elements within a structure by dispatching on type. Interpreter frequently employs Visitor to implement evaluation or optimization passes over grammar-based structures.

---

*Based on: Design Patterns (Gamma, Helm, Johnson, Vlissides, 1994)*
