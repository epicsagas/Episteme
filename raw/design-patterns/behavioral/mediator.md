# Mediator

## Essence
Mediator is a behavioral design pattern that centralizes communication between a group of objects, eliminating the need for them to reference each other directly. Instead of components holding references to every peer they might interact with, each component knows only the mediator, which coordinates all message routing and collaboration logic. This reduces a web of many-to-many connections into a star topology with the mediator at the center.

## Motivation
Consider the instrument panel of a modern automobile. The engine management system, climate control, infotainment unit, and heads-up display all need to react to shared events such as gear shifts, door openings, or low-fuel warnings. If each subsystem directly calls methods on every other subsystem, a change to the fuel gauge interface ripples through half a dozen modules. Testing any subsystem in isolation becomes nearly impossible because it is entangled with all the others.

Mediator addresses this by introducing a vehicle-coordination module that receives event notifications from subsystems and dispatches responses to the interested parties. Subsystems remain oblivious to each other's existence; they simply report state changes to the mediator and respond to directives it sends back. Adding a new subsystem requires only registering it with the mediator, not wiring it into every existing component.

## Participants
Colleague objects encapsulate domain behavior while holding a reference to the mediator through which all inter-component communication flows. The Mediator interface declares the methods colleagues use to signal events or request information. Concrete mediators implement coordination logic, storing references to colleague instances and deciding which colleagues should react to each event. The client constructs colleagues and the mediator, then registers colleagues with the mediator to complete the wiring.

## Application

**Use when:**
- a group of objects communicates in complex, many-to-many ways that make individual classes hard to reuse
- customizing interaction behavior should not require modifying the participating components
- you want to decouple components so they can be tested or deployed independently

**Prefer alternatives when:**
- component interactions are simple and stable, adding a mediator would centralize logic that is already clear
- the number of interacting objects is very small and direct coupling is manageable

## Consequences
Mediator improves adherence to the Single Responsibility Principle by extracting interaction logic from individual components and to the Open/Closed Principle by allowing new mediation strategies without changing the components. Components become more reusable because they carry no peer-specific dependencies. The risk is that the mediator itself can balloon into a monolithic god object if it accumulates too much coordination responsibility. Maintaining the mediator's internal clarity requires discipline, and tracing a message path now involves an extra indirection through the mediator rather than following direct method calls.

## Relations
Mediator is frequently compared to Facade, but Facade offers a simplified outward-facing interface to a subsystem without the subsystem being aware of it, whereas Mediator is actively known to the components it coordinates. Observer and Mediator both decouple senders from receivers: Observer does so through a publish-subscribe broadcast, while Mediator routes messages through explicit coordination logic. In practice, mediators often use Observer internally to receive notifications from colleagues. Chain of Responsibility offers yet another decoupling mechanism, routing requests sequentially rather than through a central hub.

---

*Based on: Design Patterns (Gamma, Helm, Johnson, Vlissides, 1994)*
