# Chain of Responsibility

## Essence
Chain of Responsibility is a behavioral design pattern that routes a request through a sequence of handler objects, letting each handler decide whether to process the request or pass it along to the next in line. The sender remains unaware of which handler will ultimately handle the request, and the chain can be reconfigured at runtime by adding, removing, or reordering handlers. This yields loose coupling between the request initiator and the set of potential processors.

## Motivation
Consider a logging framework that must decide where each log message ends up based on severity. A debug-level message might write to the console and stop, while a critical error should propagate through console logging, file logging, and finally an alert service that pages on-call engineers. Hard-coding this cascading logic into a single class couples all destination concerns together and makes the pipeline rigid. If a new stage such as structured JSON export gets added, every existing routing path must be revisited. Chain of Responsibility solves this by giving each destination its own handler object linked to the next, so a message flows forward until handled or the chain terminates.

A similar problem surfaces in access-control pipelines for web applications. A request passes through authentication, authorization, rate limiting, and input sanitization stages. Each stage is independently testable and replaceable, and the operations team can reorder or disable stages without touching request-handling code.

## Participants
The pattern revolves around a Handler interface that declares a method for processing requests and optionally storing a reference to the successor handler. A base handler class often provides default chaining logic, forwarding any request it cannot process to the next handler. Concrete handlers implement the actual processing for specific request categories, deciding on each invocation whether to act, forward, or both. The client assembles the chain before use and submits requests to the first handler, trusting the pipeline to route correctly.

## Application

**Use when:**
- A request must be processed by one of several candidates chosen at runtime
- the set of processing stages or their ordering changes dynamically
- you want to decouple the request sender from whichever handler ultimately responds

**Prefer alternatives when:**
- every request always requires the same single handler with no branching logic
- the processing pipeline is tiny and stable, making the abstraction unjustified overhead

## Consequences
The pattern promotes the Single Responsibility Principle by isolating each processing concern in its own class and the Open/Closed Principle by allowing new handlers without modifying existing ones. Runtime reconfigurability is a major strength: chains can be assembled differently per context. The trade-off is that a request may traverse the entire chain without being handled, which requires a fallback strategy. Debugging can also become harder because the execution path is distributed across multiple objects rather than visible in one method. When chains grow long, latency accumulates as each handler inspects the request before forwarding.

## Relations
Chain of Responsibility shares structural DNA with Decorator, since both form linked chains of objects. The difference is behavioral: Decorator wraps objects to add responsibilities transparently, while Chain of Responsibility passes requests along until one handler acts. It is frequently combined with Composite to handle recursive tree structures such as DOM event bubbling. Command can serve as the payload that travels through the chain, encapsulating request details. Mediator and Observer offer alternative approaches to routing: Mediator centralizes dispatch logic in a single object, whereas Observer broadcasts to all subscribers simultaneously rather than proceeding sequentially.

---

*Based on: Design Patterns (Gamma, Helm, Johnson, Vlissides, 1994)*
