# Strategy

## Essence
Strategy is a behavioral design pattern that defines a family of interchangeable algorithms, encapsulates each one in a separate class, and lets the context delegate execution to whichever algorithm is currently selected. The context works exclusively through a common interface, so swapping one strategy for another requires no changes to the context's code. This makes algorithm selection a runtime decision rather than a compile-time commitment.

## Motivation
Consider an e-commerce platform that calculates shipping costs using different formulas depending on the carrier, destination region, and customer loyalty tier. The checkout module could embed every formula in a massive switch statement, but each time the business adds a carrier or adjusts a rate table, the checkout code must change and be re-tested. Worse, promotions like free-shipping weekends introduce temporary strategies that clutter the module further.

Strategy addresses this by extracting each pricing formula into its own class implementing a shared cost-calculation interface. The checkout module holds a reference to a strategy object and calls its calculate method. Switching carriers, applying a promotion, or A/B testing a new algorithm is a matter of assigning a different strategy instance at runtime. The checkout logic remains untouched.

## Participants
The Context class maintains a reference to a strategy object and exposes a setter or constructor parameter for injecting a different strategy. It delegates algorithm execution to the strategy through the common interface. The Strategy interface declares the method signature that all concrete strategies implement. Concrete Strategy classes provide distinct algorithm implementations, each self-contained and testable in isolation. The client selects and configures the appropriate strategy before passing it to the context.

## Application

**Use when:**
- multiple variants of an algorithm exist and the appropriate one must be chosen at runtime
- a class contains large conditional blocks that select among algorithm variants
- you need to isolate volatile algorithm logic from stable surrounding code

**Prefer alternatives when:**
- there is only one algorithm and no realistic prospect of alternatives
- the algorithm is trivial and wrapping it in a class would add indirection without value
- the language supports first-class functions and a simple function reference achieves the same flexibility without a class hierarchy

## Consequences
Strategy aligns with the Open/Closed Principle because new algorithms are added as new classes without touching existing context or strategy code, and with the Single Responsibility Principle by isolating each algorithm in its own unit. Runtime swappability is a core strength that enables A/B testing, feature flags, and plug-in architectures. The downside is that clients must understand the available strategies well enough to choose the right one, which can push decision logic up the call stack. Introducing a strategy hierarchy for a single, stable algorithm adds unnecessary complexity and fragmentation.

## Relations
Strategy shares structural similarity with State, Bridge, and Adapter since all rely on composition and delegation to an interface. State differs by allowing its delegated objects to know about and trigger transitions to other state objects. Bridge separates abstraction from implementation so that both can vary independently, while Strategy varies only the algorithm. Template Method achieves similar goals through inheritance rather than composition: the base class defines the algorithm skeleton and subclasses override individual steps, making the structure static at compile time. Decorator wraps objects to add behavior, while Strategy swaps an object's internal algorithm entirely.

---

*Based on: Design Patterns (Gamma, Helm, Johnson, Vlissides, 1994)*
