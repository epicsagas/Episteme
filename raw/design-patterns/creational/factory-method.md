# Factory Method

## Essence
Factory Method is a creational design pattern that delegates object instantiation to a method overridden by subclasses, allowing a framework to create objects without knowing their concrete types. The parent class defines a creation hook and works with the result through a common interface, while each subclass decides which concrete class to instantiate. This preserves the parent's generic algorithm while keeping the door open for unlimited product variations.

## Motivation
A document editor ships with a core framework that manages open files, tracks revisions, and handles user interaction. The framework must instantiate document objects whenever a user opens a file, but the concrete document type depends on the application edition: a text editor creates plain-text documents, a design tool creates canvas documents, and a spreadsheet creates workbook documents. Hard-coding `new TextDocument()` inside the framework ties it to a single product and forces every new edition to fork the entire codebase. Conditional branching on an edition flag scatters `instanceof` checks throughout the framework, and adding a new document type means editing every branch.

Factory Method resolves this by letting the framework call an abstract `createDocument()` method it does not implement. Each application edition subclasses the framework and overrides the method to return the appropriate document type. The framework's remaining logic operates exclusively on the abstract `Document` interface, unaware of which concrete class lives behind it. Adding a `PresentationDocument` later requires only a new subclass override, not a single line changed in the framework.

## Participants
The product interface declares the operations all creatable objects must support. Concrete product classes implement this interface with variant-specific behavior. The creator class contains the core algorithm and declares the factory method, which may be abstract or provide a default implementation. Concrete creator subclasses override the factory method to return a specific concrete product. Importantly, the creator's primary responsibility is its own business logic; the factory method is a secondary hook that keeps the creator decoupled from direct instantiation. The method can also return cached instances or pull objects from a pool, something a constructor can never do.

## Application

**Use when:**
- A class cannot anticipate which concrete type it must create, and subclasses should make that decision
- A framework needs to let users extend internal components by inheriting rather than modifying framework source
- Object pooling or caching makes returning existing instances preferable to always constructing fresh ones
- Multiple related products share a common interface but each requires distinct initialization logic

**Prefer alternatives when:**
- The concrete type is always known at compile time (direct construction is simpler and faster)
- Product families must be created in coordinated groups (Abstract Factory is a better fit)
- Construction requires many optional steps (Builder offers more flexibility)

## Consequences
Factory Method eliminates hard dependencies on concrete classes, which makes the system easier to extend and test. A unit test can subclass the creator and return a mock product without touching production code. The pattern also centralizes instantiation in one overridable method, supporting the Single Responsibility Principle by keeping creation logic separate from business logic. The cost is a growing class hierarchy: every new product variant typically demands a new creator subclass. When the number of variants is small, this overhead is negligible, but it can become unwieldy in systems with dozens of product types unless combined with a parameterized factory or a registry. Because the method can return cached objects, it enables performance optimizations impossible with plain constructors.

## Relations
Factory Method is the simplest of the creational family and often evolves into Abstract Factory when a system accumulates several related factory methods that should be grouped under one interface. Builder shares the delegation-to-subclass idea but focuses on multi-step assembly rather than a single creation call. Prototype offers an alternative mechanism: instead of subclassing to vary the product, the creator clones a prototype object, trading inheritance depth for object composition. Template Method and Factory Method share a structural similarity, since both define a skeleton algorithm and let subclasses fill in a specific hook. Iterator sometimes relies on Factory Method when collection subclasses must return compatible iterator types.

---

*Based on: Design Patterns (Gamma, Helm, Johnson, Vlissides, 1994)*
