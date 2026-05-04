# Adapter

## Essence
Adapter is a structural design pattern that bridges incompatible interfaces by wrapping one object and translating its method signatures into the form another object expects. The adapter implements the interface the client relies on and delegates calls to the wrapped service, converting data formats, parameter orders, or invocation semantics along the way. Client code sees only the interface it understands, unaware that translation is happening behind the scenes.

## Motivation
A weather-forecasting application consumes data from a government meteorological service that returns measurements in an older XML-based protocol. The application's analytics module, written years later, expects data delivered through a modern JSON-based interface with metric units. Rewriting the analytics module to understand XML would pull legacy format concerns into cleanly separated business logic, and lobbying the government agency to modernize its API is unrealistic. The team needs the analytics module to work with the existing feed without modifying either side.

An adapter class implements the JSON-based interface the analytics module expects while internally maintaining a reference to the legacy XML service. When the analytics module requests a temperature reading, the adapter calls the XML service, parses the response, converts Fahrenheit to Celsius, and returns a value in the format the caller understands. Neither the analytics module nor the legacy service knows the adapter exists, and swapping to a new data source later requires only a new adapter, not changes in the analytics code.

## Participants
The target interface defines the method signatures the client expects to call. The adaptee is the existing class with an incompatible interface that holds the needed functionality. The adapter class implements the target interface and holds a reference to an adaptee instance, translating each target method call into one or more adaptee method calls with the necessary data conversions. The client interacts exclusively with the target interface, receiving an adapter instance at runtime. This composition-based approach works in any language. A class-adapter variant uses multiple inheritance (available in C++ and similar languages) to inherit from both the target and the adaptee simultaneously, which is more compact but less flexible.

## Application

**Use when:**
- An existing class provides the right behavior but the wrong method signatures for a client that cannot be modified
- A legacy subsystem must integrate with newer code without touching either side's public API
- Multiple third-party libraries offer similar functionality through different interfaces and the application needs a uniform access layer

**Prefer alternatives when:**
- The service class can be modified directly (change the interface and avoid the indirection)
- The mismatch is minor enough that a simple wrapper function or extension method suffices
- A completely new abstraction over a subsystem is needed rather than translating between two existing ones (Facade applies)

## Consequences
Adapter preserves existing code by inserting a thin translation layer instead of forcing rewrites, which reduces risk in systems where the adaptee is fragile or externally maintained. The pattern supports the Open/Closed Principle because new adapters can be added without changing client or service code. However, each adapter adds a class and a level of indirection that can obscure the call chain during debugging. Performance-sensitive paths may suffer from the extra delegation, though this cost is usually negligible. When many adapters accumulate for slight interface variations, the codebase can feel cluttered, and developers may question whether a shared base interface or a broader refactoring would be more appropriate.

## Relations
Adapter is often confused with Facade, but Facade simplifies a complex subsystem behind a new, easier interface, while Adapter makes two existing interfaces compatible without simplifying either. Bridge shares the wrapping structure but is designed upfront to separate an abstraction from its implementation, whereas Adapter is a retrofit for interfaces that were never meant to work together. Decorator also wraps an object, yet Decorator adds new behavior while preserving the same interface, while Adapter changes the interface while preserving existing behavior. Proxy maintains the same interface as the service and controls access to it, whereas Adapter deliberately presents a different interface. Strategy, State, and Adapter all use composition, but each solves a fundamentally different problem: behavior selection, state-driven behavior, and interface translation, respectively.

---

*Based on: Design Patterns (Gamma, Helm, Johnson, Vlissides, 1994)*
