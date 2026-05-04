# Bridge

## Essence
Bridge is a structural design pattern that splits a class hierarchy into two independent dimensions, an abstraction and its implementation, connected through composition rather than inheritance. The abstraction holds a reference to an implementation object and delegates platform-specific or detail-level work to it, allowing each side to evolve independently. New abstractions or new implementations can be added without touching the other hierarchy.

## Motivation
A payment processing platform supports multiple transaction channels (in-store terminals, mobile wallets, online checkout) and multiple regional backends (European SEPA, US ACH, Asian wire transfer). Modeling every combination with inheritance produces a separate subclass for each pair: `InStoreSepaProcessor`, `MobileAchProcessor`, `OnlineWireProcessor`, and so on. Adding a new channel forces new subclasses for every existing backend, and adding a new backend forces new subclasses for every existing channel. The class graph explodes multiplicatively, and shared logic between `InStoreSepaProcessor` and `InStoreAchProcessor` is duplicated because inheritance cannot express "same channel, different backend" cleanly.

Bridge resolves this by extracting the backend dimension into its own hierarchy. A `PaymentBackend` interface declares methods like `authorize()` and `capture()`, with concrete implementations for each region. The `PaymentChannel` abstraction (in-store, mobile, online) holds a reference to a `PaymentBackend` and delegates the actual money movement to it. Adding a new channel means one new abstraction class that works with all existing backends. Adding a new backend means one new implementation class that works with all existing channels. The two hierarchies grow additively rather than multiplicatively.

## Participants
The implementation interface declares the low-level operations that concrete implementations must provide. Concrete implementation classes contain platform-specific or region-specific code behind that interface. The abstraction class defines high-level operations and maintains a reference to an implementation object, forwarding relevant calls to it. Refined abstractions extend the base abstraction with additional operations or default behaviors, still delegating to the implementation for low-level work. The client configures an abstraction with the desired implementation at runtime, choosing the pairing without the framework dictating a fixed combination.

## Application

**Use when:**
- A class must be extended across two or more independent dimensions (platform and feature set, data format and storage engine, UI framework and business logic)
- Runtime switching between implementations is required (toggling between a production database and an in-memory store during testing)
- Sharing an implementation among several objects avoids duplication (multiple abstractions backed by the same data source)

**Prefer alternatives when:**
- The class has only one varying dimension (plain inheritance is sufficient)
- The implementation is tightly coupled to the abstraction and will never vary independently (Adapter may be simpler if the goal is just making one interface work with another)
- The hierarchy is small and unlikely to grow (the added indirection is not justified)

## Consequences
Bridge promotes the Open/Closed Principle because new abstractions and new implementations can be introduced without modifying existing code. It also supports the Single Responsibility Principle by separating high-level policy from low-level detail. The composition link enables implementation swapping at runtime, which is useful for mode switches, testing mocks, and feature flags. The trade-off is increased type count: every dimension gets its own interface and concrete classes. For small systems with a single varying axis, this overhead is unnecessary. The pattern also requires disciplined design upfront; retrofitting Bridge into an existing deep inheritance hierarchy can be a significant refactoring effort, though the long-term maintainability gain usually justifies the cost.

## Relations
Bridge and Adapter share the same composition mechanism but differ in intent and timing. Bridge is planned during design to keep hierarchies independent, while Adapter is applied after the fact to reconcile incompatible interfaces. Strategy and State have identical structural diagrams: an abstraction delegates to a swappable implementation. The difference is that Strategy swaps algorithms within a single context, State swaps behavior based on an object's lifecycle stage, and Bridge swaps platform-specific mechanisms behind a stable abstraction. Abstract Factory often pairs with Bridge because the factory can instantiate the correct implementation for a given abstraction, ensuring the two are compatible. Builder's director-builder pairing mirrors the abstraction-implementation split of Bridge, though Builder focuses on construction sequencing rather than runtime delegation.

---

*Based on: Design Patterns (Gamma, Helm, Johnson, Vlissides, 1994)*
