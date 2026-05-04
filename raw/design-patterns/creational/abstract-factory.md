# Abstract Factory

## Essence
Abstract Factory is a creational design pattern that produces entire families of related objects through a shared interface, shielding client code from knowing which concrete classes get instantiated. Each concrete factory encapsulates all the `new` operator calls for one complete product variant, guaranteeing that objects from the same family are always created together. Switching the entire product line becomes a single-line change: swap one factory for another, and every downstream call automatically receives the right variant.

## Motivation
Consider a mapping application that renders vector tiles on both a desktop OpenGL backend and a browser WebGL frontend. Every frame needs a shader program, a vertex buffer, and a texture atlas, and each backend provides its own concrete implementations. If a developer accidentally pairs an OpenGL shader with a WebGL texture, the renderer crashes at draw time with cryptic driver errors. Hard-coding every valid combination in conditional branches scatters `if` statements across dozens of modules, and adding a third backend (say, a Vulkan mobile renderer) means touching all those branches simultaneously.

Abstract Factory eliminates this fragility by letting each backend ship its own factory class. The desktop module supplies `OpenGLFactory`, the browser supplies `WebGLFactory`, and the mobile module later contributes `VulkanFactory` with no changes to existing code. Client modules receive a factory once at startup and request shaders, buffers, and textures exclusively through its methods, confident that every object belongs to the same rendering family.

## Participants
An abstract product interface declares the contract for each category the family covers (shader, buffer, texture). Concrete product classes implement these interfaces for a specific variant. An abstract factory interface lists a creation method for every product category. Each concrete factory implements all those methods, returning the variant-specific products that belong together. Client code holds a reference to the abstract factory and calls its methods, never referencing concrete classes directly. Because every product the client receives originates from the same concrete factory, cross-variant mismatches are impossible at runtime.

## Application

**Use when:**
- A system must support multiple product families that must not be mixed (rendering backends, database dialects, UI themes across operating systems)
- Concrete product classes should remain hidden behind interfaces so new variants can ship without recompiling existing clients
- A group of factory methods on one class is diluting its primary responsibility, suggesting extraction into a dedicated factory object

**Prefer alternatives when:**
- Only a single product type is needed, and family consistency is not a concern (Factory Method suffices)
- Construction requires many optional parameters or step-by-step assembly (Builder is more appropriate)
- The product count is small and unlikely to grow (direct construction is simpler)

## Consequences
The pattern enforces family-level consistency at the cost of a larger type surface: every new product category adds another abstract interface, another method to every concrete factory, and a new concrete class per variant. This overhead pays off when variants are numerous or when third-party plug-ins supply their own families. For two variants and two product types, the indirection may outweigh the benefit. Adding a new variant is straightforward (implement all product interfaces and build a new concrete factory), but adding a new product category forces changes to every existing factory, which can cascade through large codebases. Abstract Factory also hides construction complexity behind a clean boundary, which simplifies testing: a test suite can inject a factory that returns lightweight stubs or mocks instead of expensive real resources.

## Relations
Factory Method and Abstract Factory share the same spirit of separating creation from usage, but Factory Method relies on inheritance (subclasses override a single creation hook) whereas Abstract Factory relies on composition (client code holds a factory object). Builder also isolates construction logic, yet Builder constructs one complex product through sequential steps while Abstract Factory returns multiple products immediately. Prototype can replace the factory internals: instead of calling constructors, a concrete factory might clone pre-registered prototype objects, which is faster when initialization is expensive. Facade sometimes pairs with Abstract Factory because a facade can internally use a factory to hide which subsystem objects it orchestrates. Bridge benefits from the pattern when each abstraction refinement needs a matching family of implementations. Concrete factories are frequently implemented as Singleton instances since only one instance per variant is necessary.

---

*Based on: Design Patterns (Gamma, Helm, Johnson, Vlissides, 1994)*
