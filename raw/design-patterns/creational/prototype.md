# Prototype

## Essence
Prototype is a creational design pattern that clones existing objects instead of constructing new ones from scratch, enabling code to duplicate complex instances without knowing their concrete classes. Each cloneable object implements a method that returns a copy of itself, preserving the original's state at the moment of duplication. This approach sidesteps the need for subclass hierarchies dedicated solely to initialization and works even when the concrete type is hidden behind an interface.

## Motivation
A terrain-generation engine for a strategy game places thousands of trees, rocks, and buildings on a map at startup. Each environmental object carries randomly generated properties: position, scale, health, and a reference to a shared mesh. Constructing every object through its constructor requires passing dozens of parameters, and the generation algorithm should not care whether it is placing an oak tree or a ruined tower, only that the object supports `place(x, y)` and `render()`. Repeating the full initialization for each instance is wasteful, especially when most objects share the same mesh and differ only in transform coordinates.

Prototype solves this by pre-building one fully configured instance of each object type and registering it in a lookup table. The terrain generator requests a clone of the "oak tree" prototype, sets its position, and moves on. The clone carries all the expensive defaults (mesh, collision shape, shader parameters) without recomputing them. When a new environmental asset is added as a plug-in, the plug-in registers its own prototype at load time, and the generator works with it transparently through the common clone interface.

## Participants
A prototype interface declares a cloning operation, often named `clone` or `copy`. Each concrete prototype implements this method by creating a new instance and copying its own field values into it, handling deep versus shallow copying as appropriate. A prototype registry (sometimes called a prototype manager) stores named prototypes in a dictionary, letting clients request clones by string key or enum without knowing the concrete class. The client asks the registry for a prototype, receives a fresh copy, and then customizes it as needed. The registry itself can be populated at startup, from configuration files, or dynamically by plug-ins.

## Application

**Use when:**
- Code must create copies of objects whose concrete classes are unknown or hidden behind interfaces
- Initializing an object is expensive (large data structures, parsed configurations, loaded assets) and a pre-built template can be duplicated cheaply
- Subclasses proliferate solely to provide different initialization presets, which could instead be represented as distinct prototype instances
- Objects must be saved and restored to a prior state without relying on serialization

**Prefer alternatives when:**
- Objects contain circular references or complex resource handles that make deep copying error-prone (Memento may be safer)
- Only a single shared instance is needed (Singleton applies)
- Construction is trivial and cloning adds no value over a plain constructor

## Consequences
Prototype decouples client code from concrete classes and eliminates repetitive initialization logic. Adding a new variant requires only registering a new prototype instance, not defining a new subclass. Cloning is fast when the cost of copying fields is lower than the cost of computing initial values from scratch, which is often the case for objects that parse files or allocate large buffers during construction. The main complication arises with deep copying: reference-type fields may need recursive cloning, and shared references must be handled carefully to avoid unintended duplication. Circular references demand special treatment, usually involving a registry of already-cloned objects to break cycles. Despite these edge cases, Prototype remains one of the lightest creational patterns in terms of class count.

## Relations
Factory Method creates objects through inheritance, while Prototype creates them through cloning, a composition-based approach. Abstract Factory can use Prototype internally: instead of calling constructors, each factory method clones a registered prototype, combining family-level consistency with fast duplication. Builder constructs objects step by step, whereas Prototype duplicates a fully formed instance in one operation. Composite and Decorator benefit greatly from Prototype because cloning a composite tree or a decorated chain is far simpler than reassembling it through construction code. Memento addresses a related problem (capturing state) but focuses on rollback rather than creating independent copies, and Memento avoids cloning issues with external resources by working with snapshots instead of live objects.

---

*Based on: Design Patterns (Gamma, Helm, Johnson, Vlissides, 1994)*
