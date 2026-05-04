# Facade

## Essence
Facade is a structural design pattern that provides a simplified, high-level interface to a complex subsystem, shielding clients from the internal classes, configuration steps, and dependency management the subsystem requires. The facade does not encapsulate or hide the subsystem; it merely offers a convenient entry point that covers the most common use cases. Clients can still reach the subsystem directly when they need fine-grained control.

## Motivation
A machine-learning inference pipeline involves five stages: loading a model from a remote store, deserializing weights into GPU memory, preprocessing input tensors, running the forward pass, and post-processing predictions into human-readable labels. Each stage is managed by a different library with its own configuration object, error-handling convention, and thread-safety model. An application that needs only a single `predict(image)` call must wire all five stages together, learn three different error-handling strategies, and ensure resources are released in the correct order. Every new application that integrates the pipeline duplicates this wiring, and any change to the pipeline's internals ripples across every call site.

A facade class called `InferenceService` exposes a single `predict(image)` method. Internally, it orchestrates the model loader, the tensor preprocessor, the inference engine, and the result formatter, handling retries, resource cleanup, and thread synchronization. Applications call one method and receive a prediction. When a team needs to customize the preprocessing stage, they bypass the facade and interact with the subsystem directly, falling back to fine-grained control for that specific scenario.

## Participants
The facade class knows which subsystem classes to instantiate and in what order to call them. It translates simple incoming requests into the corresponding subsystem operations, managing any necessary configuration or state along the way. The subsystem consists of the many classes that perform the actual work; these classes have no knowledge of the facade and operate independently. Additional facades can be introduced when the primary facade grows too large, each targeting a different subset of subsystem functionality. The client invokes methods on the facade rather than on individual subsystem objects.

## Application

**Use when:**
- A subsystem has grown complex enough that initializing and coordinating it requires substantial boilerplate
- Multiple client modules share the same subsystem interaction pattern, suggesting a shared convenience layer
- Layered architecture demands a clear entry point per subsystem, limiting cross-layer dependencies to facade interfaces

**Prefer alternatives when:**
- The subsystem is already simple and a facade would add unnecessary indirection
- Clients need full access to every subsystem feature and a simplified interface would be too restrictive
- The goal is interface translation rather than simplification (Adapter applies)

## Consequences
Facade reduces coupling between clients and subsystem internals, which means subsystem refactoring rarely breaks client code as long as the facade's contract remains stable. New developers can integrate the subsystem by reading the facade's documentation instead of studying dozens of internal classes. The pattern adds minimal overhead because the facade is a thin coordination layer with no heavy logic of its own. The risk is that the facade itself can become a god object if it accumulates methods for every subsystem operation; at that point, splitting into multiple targeted facades or using a Mediator to decouple subsystem components is advisable. Facade does not prevent clients from accessing the subsystem directly, so the simplified interface is a convenience, not a constraint.

## Relations
Facade and Adapter both sit between a client and another layer, but Adapter reconciles two incompatible interfaces while Facade creates a simpler interface over a complex subsystem that is already compatible. Mediator and Facade both reduce direct coupling between components, though Mediator centralizes bidirectional communication between peers while Facade provides a one-directional simplification for the caller's benefit. Proxy wraps a single object and maintains its interface, whereas Facade orchestrates many objects and presents a new, simpler interface. Abstract Factory can work alongside Facade when the facade needs to create subsystem objects without exposing their concrete types. Facade is frequently implemented as a Singleton because only one coordinating instance is typically needed per subsystem.

---

*Based on: Design Patterns (Gamma, Helm, Johnson, Vlissides, 1994)*
