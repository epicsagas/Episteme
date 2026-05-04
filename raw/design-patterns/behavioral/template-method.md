# Template Method

## Essence
Template Method is a behavioral design pattern that defines the skeleton of an algorithm in a base class, deferring specific steps to subclasses while preserving the overall algorithm structure. The base class declares abstract or hook methods for the variable steps and a final template method that calls those steps in a fixed sequence. Subclasses customize behavior by overriding individual steps without altering the algorithm's flow.

## Motivation
Imagine a data-ingestion pipeline that fetches files from heterogeneous sources such as S3 buckets, SFTP servers, and internal message queues. Regardless of source, every ingestion follows the same stages: connect, authenticate, download, validate checksums, parse into a canonical format, and write to the data lake. Only the connect, authenticate, and download steps differ across sources, yet without Template Method each pipeline implementation duplicates the shared validation, parsing, and writing logic.

By placing the invariant stages in a base class and declaring the variable stages as abstract methods, the common logic is written once and inherited. Adding a new data source means subclassing and implementing three methods. The template method in the base class guarantees that validation always follows download and parsing always follows validation, eliminating the risk that a developer forgets a step.

## Participants
The Abstract Class houses the template method, which orchestrates the algorithm by calling a sequence of primitive operations. Some of these operations are abstract, forcing subclasses to supply implementations, while others offer default behavior or serve as hooks with empty bodies that subclasses may optionally override. Concrete Classes extend the abstract class and implement the abstract operations, customizing the algorithm's variable portions. The template method itself is marked as final or non-overridable so that subclasses cannot disrupt the algorithm's structure.

## Application

**Use when:**
- multiple classes share an algorithmic structure with only isolated steps differing between them
- you want to enforce a fixed execution order for algorithm steps while allowing customization of individual steps
- common code across similar classes should be consolidated into a shared base

**Prefer alternatives when:**
- the algorithm has no meaningful invariant structure and every step varies independently
- composition-based approaches like Strategy provide the flexibility you need without a class hierarchy

## Consequences
Template Method promotes code reuse by pulling shared logic into the base class and gives the base class control over which extension points are available. It supports the Hollywood Principle: the base class calls the subclass, not the other way around. The chief risk is that the Liskov Substitution Principle can be violated if a subclass suppresses a step by overriding it with an empty body, subtly breaking the algorithm's contract. Adding new steps to the template method forces changes to every existing subclass, so the skeleton must be stable before the pattern pays off. In languages without a final keyword, developers can accidentally override the template method itself.

## Relations
Factory Method is often described as a specialized application of Template Method where the template creates objects. Strategy offers a composition-based alternative: instead of varying behavior through inheritance and overridden methods, the context delegates to interchangeable strategy objects at runtime. This makes Strategy more flexible when algorithm variants must be swapped dynamically, while Template Method is simpler when the structure is fixed at compile time. The two can be combined: a template method may call a strategy for one of its steps to gain runtime flexibility at that point.

---

*Based on: Design Patterns (Gamma, Helm, Johnson, Vlissides, 1994)*
