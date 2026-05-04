# Observer

## Essence
Observer is a behavioral design pattern that establishes a one-to-many dependency between objects so that when one object changes state, all its dependents are notified and updated automatically. Rather than having subjects poll their dependents or hardcode notification logic, the pattern formalizes a subscription mechanism where interested parties register themselves. This decouples the event source from its consumers, allowing either side to evolve without the other's knowledge.

## Motivation
Consider a real-time stock-trading dashboard that displays price tickers, portfolio valuations, and alert badges. When a trade executes and a price updates, every widget showing that symbol must refresh immediately. Embedding direct calls to each widget inside the trading engine would tie the engine's core logic to the specifics of the UI layer, and every new widget added to the dashboard would require modifying the engine's notification code.

Observer solves this by having the trading engine maintain a list of registered listeners. Widgets subscribe to the symbols they display, and the engine broadcasts price-change events to all subscribers. A new alert widget joins the system simply by registering as a listener, with zero changes to the trading engine. The engine neither knows nor cares how many widgets are listening.

## Participants
The Subject, also called the publisher, maintains a collection of observer references and provides methods to attach and detach observers. When the subject's state changes in a way that interests observers, it iterates through the collection and invokes each observer's update method. The Observer interface declares the notification contract that concrete observers must fulfill. Concrete observers perform application-specific logic in response to notifications, often querying the subject for additional details. The client wires observers to subjects at runtime, establishing the subscription relationships.

## Application

**Use when:**
- state changes in one object require reactions from an unknown or dynamically changing set of other objects
- an event source should remain unaware of which and how many listeners exist
- you need to support dynamic subscription and unsubscription at runtime

**Prefer alternatives when:**
- the set of dependents is fixed and small, making a direct method call simpler and more readable
- notification ordering matters and the unpredictability of broadcast order would introduce bugs, in which case a more explicit dispatch mechanism is appropriate

## Consequences
Observer supports the Open/Closed Principle because new observer types can be introduced without modifying the subject, and it enables runtime flexibility through dynamic subscription. The pattern's main strength is the complete decoupling of event producers from event consumers. However, this same decoupling can make control flow harder to trace, since there is no visible call from the observer back to the subject in static analysis. Unintended memory leaks can occur if observers are not properly unsubscribed, because the subject's reference list keeps them reachable. Notification order is typically undefined, so observers must not depend on a specific sequencing unless the subject documents guarantees.

## Relations
Observer differs from Mediator in communication topology: Observer broadcasts to all subscribers simultaneously, while Mediator routes messages selectively through a central coordinator. Chain of Responsibility offers sequential dispatch along a handler chain, stopping when one handler acts, contrasting with Observer's fan-out to all listeners. Command can serve as the payload delivered through an Observer notification, encapsulating the event details. In practice, Mediator implementations often rely on Observer internally to decouple mediator-to-component notifications.

---

*Based on: Design Patterns (Gamma, Helm, Johnson, Vlissides, 1994)*
