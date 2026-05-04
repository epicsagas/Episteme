# Introduce Local Extension

## Motivation

Introduce Local Extension addresses the problem of a third-party or framework class that almost meets your needs but falls short by several methods. Unlike Introduce Foreign Method, which parks a single helper on a client class, this technique creates a dedicated extension — either a subclass or a wrapper — that houses all the supplementary behavior in one coherent location. The extension object is usable wherever the original class is expected, because it preserves the full original interface while adding the missing operations.

The technique becomes necessary when scattered foreign methods begin to outweigh the convenience of their ad-hoc placement. A date library that lacks business-day arithmetic, quarter-boundary calculations, and fiscal-year logic forces each of these computations into whatever client class happens to need it first. The result is a distributed mess of static helpers that no single class owns and no developer can discover without searching the entire codebase.

## Mechanics

1. Choose the extension strategy. Subclassing is simpler: the extension inherits every method and field automatically, and language-level polymorphism allows the subclass to substitute for the original wherever the type is expected. However, if the library class is final or if you need to extend a class that is already instantiated by framework code you cannot control, subclassing is blocked. In that case, use a wrapper: a new class that holds an instance of the library class as a private field and delegates every original method to it.

2. Create the extension class. For a subclass, declare it extending the library class. For a wrapper, declare a field of the library type and implement delegation methods for every public method the library exposes. Use your IDE's delegate-generation feature to avoid manual transcription errors.

3. Implement constructors. Provide a constructor that mirrors each of the library class's constructors, passing arguments through via super or by initializing the wrapped instance. Add a converting constructor or factory method that accepts an existing library object and wraps or upcasts it, enabling seamless adoption at boundaries where library code returns the base type.

4. Move each foreign method from its current client-class location into the extension. These methods no longer need the library object as a parameter because they operate on `this` (subclass) or on the wrapped field (wrapper). Adjust the method signatures accordingly.

5. Replace usages of the library type with the extension type at every call site that needs the supplementary behavior. Where library code returns the base type, apply the converting constructor to promote it to the extension. Test after each replacement.

## Indications

**Signs suggesting this refactoring:**
- Multiple foreign methods targeting the same library class have accumulated across different client classes
- Incomplete Library Class is the dominant smell: the library provides the foundation but your domain requires operations the library authors never anticipated
- Client classes contain static utility methods that conceptually belong to the library type but cannot be added to it

**When to avoid:**
- Only one or two foreign methods exist — Introduce Foreign Method is sufficient and avoids the overhead of a new class
- The wrapper variant would need to delegate dozens of methods, making the wrapper brittle against library version upgrades that add or change the interface
- Your codebase already uses a utility or helper class for the library, and consolidating there would be simpler than introducing a new type hierarchy

## Trade-offs

Subclassing is the leaner variant: no delegation boilerplate, automatic interface compatibility, and natural use in polymorphic contexts. Its limitation is rigidity — a final class cannot be subclassed, and some frameworks instantiate classes internally, bypassing your subclass. Wrapping avoids both constraints but pays a heavy mechanical price: every public method of the wrapped class must be delegated, and each library update that adds or renames methods breaks the wrapper silently if not updated in lockstep. Wrappers also break object identity — the wrapped instance and the wrapper are different objects, which matters for identity-based collections and reference comparisons.

Both variants add a type to the codebase that developers must learn. The extension class must be discoverable: named clearly, documented as extending a specific library class, and imported wherever needed. When the extension is small and focused, this overhead is manageable. When it grows large, it risks becoming a parallel utility class with its own cohesion problems, at which point reconsidering the boundary between extension logic and domain logic becomes necessary.

## Connections

Introduce Local Extension is the scaled-up successor to Introduce Foreign Method — when foreign methods multiply, they graduate into a local extension. Both address the Incomplete Library Class smell. The subclass variant relates to the Decorator Pattern in structure, though decorators add cross-cutting behavior while extensions add domain-specific operations. The Adapter Pattern is a neighbor: adapters translate between incompatible interfaces, while extensions supplement a compatible one. Extract Class is conceptually related in that both create a new structural unit to house behavior that does not belong in its current location, though the motivations differ — extraction splits responsibilities, extension supplements an unmodifiable interface.

---

*Based on: Refactoring (Fowler, 1999)*
