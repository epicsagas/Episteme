# Singleton

## Essence
Singleton is a creational design pattern that restricts a class to a single instance while providing a global access point to that instance. The class hides its constructor and exposes a static method that creates the object on the first call and returns the cached object on every subsequent call. All consumers share the same state, which is useful for coordinating access to shared resources like connection pools or configuration managers.

## Motivation
A distributed logging service needs exactly one log-writer process per application to sequence messages correctly and prevent overlapping writes to the same output file. If two independent modules each instantiate their own writer, log entries from concurrent requests interleave unpredictably, and the resulting output is corrupt. Passing a single writer through every function signature is possible but burdens the entire call chain with an extra parameter that is rarely relevant to intermediate layers. A global variable solves the sharing problem but offers no protection against accidental reassignment, and it cannot guarantee lazy initialization.

Singleton addresses both concerns. The writer class hides its constructor, so no caller can create a second instance through normal means. A static method returns the sole instance, initializing it lazily the first time it is requested. Every module that calls the method receives the same object, ensuring consistent write ordering without polluting function signatures. Thread-safe implementations use synchronization primitives to guarantee uniqueness even when multiple threads request the instance concurrently.

## Participants
The singleton class contains a private static field that holds the sole instance and a public static method (often named `getInstance`) that lazily creates the instance if absent and returns it. The constructor is private or protected, blocking external instantiation. In some implementations, the class also overrides clone prevention and serialization hooks to block alternative paths that could produce a second instance. Clients access the singleton exclusively through the static method, never through constructors.

## Application

**Use when:**
- Exactly one shared instance must coordinate a resource across the entire application (database connection pool, hardware driver handle, configuration registry)
- Global access to that instance must be controlled more tightly than a bare global variable allows
- Lazy initialization is desired so the resource is not allocated until it is actually needed

**Prefer alternatives when:**
- Multiple independent consumers need different instances of the same class (use dependency injection instead)
- The class has significant side effects that make unit testing difficult (inject a mockable interface)
- Statelessness is possible; a class with only static methods may serve the same purpose without the singleton baggage

## Consequences
Singleton guarantees a single shared instance and eliminates the overhead of passing references through deep call chains, which simplifies code that genuinely needs one shared resource. Lazy initialization defers costly setup until the first request, avoiding unnecessary work at application startup. However, the pattern carries significant drawbacks. The hidden global state makes reasoning about program behavior harder because any module can mutate shared data at any time. Unit testing suffers because the private constructor and static access point make it difficult to substitute mocks without reflection or test-specific hooks. Multithreaded environments demand careful synchronization to prevent race conditions during lazy initialization. The pattern also violates the Single Responsibility Principle by simultaneously managing instance lifecycle and providing business functionality. Overuse of Singleton often signals an architectural issue: excessive coupling that should instead be resolved through dependency injection frameworks.

## Relations
Facade is frequently implemented as a Singleton because applications typically need only one facade instance coordinating a subsystem. Abstract Factory, Builder, and Prototype factories are sometimes implemented as Singletons when only one factory instance per variant is needed, which avoids redundant factory objects. Flyweight shares a conceptual similarity in that both patterns centralize access to shared state, but Flyweight manages multiple immutable shared objects while Singleton manages exactly one mutable instance. Monostate is a lesser-known variant where all instances share the same static fields but instantiation is unrestricted, offering Singleton-like behavior without restricting construction. Dependency injection frameworks can replace most Singleton use cases by managing object lifecycle in a container, providing the same single-instance guarantee without static coupling.

---

*Based on: Design Patterns (Gamma, Helm, Johnson, Vlissides, 1994)*
