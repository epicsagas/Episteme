# Duplicate Observed Data

## Motivation

Domain data sometimes resides inside GUI classes, mixed together with event handlers, layout logic, and rendering code. This entanglement makes the business logic difficult to test because any test must instantiate the entire UI framework, and it forces duplication when multiple views, such as a desktop client and a web dashboard, must display the same underlying data. The root cause is a failure to separate the presentation layer from the domain model, violating the Single Responsibility Principle at an architectural level.

This refactoring extracts the business data into a dedicated domain class while keeping the GUI class as a passive observer. The domain class owns the data and notifies registered observers whenever it changes, allowing any number of independent views to react without knowing about each other. This separation enables unit testing of domain logic without a UI harness and supports multiple interface technologies against the same business objects.

## Mechanics

1. Identify every field in the GUI class that holds domain data rather than presentation state. Apply Self Encapsulate Field to each one, creating getters and setters that mediate all access.
2. Modify every GUI event handler that writes to these fields so it goes through the new setters instead of assigning directly. This establishes a single point of control for each value.
3. Create a new domain class and copy the relevant fields and their getters and setters into it. The domain class should have no dependency on any UI framework.
4. Implement the Observer pattern between the domain class and the GUI class. Add an observer registration mechanism to the domain class, give the GUI class an update method, and register the GUI instance as an observer during construction. Domain setters should invoke the notification method after updating their state, and GUI setters should forward changes to the domain object rather than modifying local copies.

## Indications

**Signs suggesting this refactoring:**
- Business rules are implemented inside button click handlers or window event callbacks, making them impossible to exercise without launching the GUI.
- The same calculation or validation appears in multiple presentation classes because the domain logic was never extracted.
- A need to support a new interface, such as a command-line tool or a REST API, reveals that all useful data is trapped inside window classes.

**When to avoid:**
- In web applications where server-side objects do not persist across requests, the classic Observer mechanism is impractical. The underlying principle of separating domain from presentation still applies, but the synchronization mechanism should be adapted, for instance through a shared database or a message bus.
- The GUI class contains only trivial display logic with no meaningful business rules, and the cost of introducing a separate domain class outweighs the benefit.

## Trade-offs

Extracting domain data into an observable class pays a significant upfront cost in plumbing: observer registration, notification dispatch, and careful handling of update cycles to avoid infinite recursion. For simple forms with minimal logic, this ceremony can double the code volume without delivering proportional value. However, as soon as a second consumer of the same data appears, the investment pays for itself by eliminating duplication and establishing a single source of truth. Testing is the other major beneficiary: domain classes can be instantiated and verified in milliseconds without waiting for a UI framework to initialize. The trade-off hinges on whether the domain logic is complex or multi-consumer enough to justify the observer infrastructure.

## Connections

This refactoring relies on Self Encapsulate Field as a preparatory step. It is a specialized form of Extract Class applied at the boundary between layers. The Observer pattern provides the synchronization mechanism, while Model-View-Controller and Model-View-Presenter architectures formalize the resulting separation. The Feature Envy smell often appears alongside this problem, when GUI classes spend more time manipulating domain data than managing presentation concerns. Extract Method can further decompose the domain class after extraction.

---

*Based on: Refactoring (Fowler, 1999)*
