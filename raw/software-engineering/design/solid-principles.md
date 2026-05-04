# SOLID Principles

## Statement
SOLID is a mnemonic for five object-oriented design guidelines -- Single Responsibility, Open/Closed, Liskov Substitution, Interface Segregation, and Dependency Inversion -- that collectively promote modularity, extensibility, and maintainability in software systems. Each principle addresses a specific symptom of poor design: rigidity, fragility, immobility, or viscosity. Applied together, they guide developers toward architectures where changes are localized, components are replaceable, and dependencies flow inward toward stable abstractions.

## Origin
Robert C. Martin synthesized these five principles in the late 1990s and early 2000s, publishing them in his 2002 book "Agile Software Development: Principles, Patterns, and Practices." The individual ideas draw on earlier academic work: Barbara Liskov formulated the substitution principle in 1987, Bertrand Meyer articulated the open/closed idea in 1988, and the dependency inversion concept appeared in Martin's earlier articles on dependency management. Martin's contribution was recognizing their synergy, grouping them under the SOLID acronym, and connecting them to agile development practices.

## Software Implications
The Single Responsibility Principle (SRP) states that a module should have exactly one reason to change. A class that both parses CSV files and sends email notifications will change when the CSV format changes or when the email service API changes -- two independent business drivers. Splitting these concerns into separate classes isolates each axis of change and prevents unrelated modifications from introducing regressions in the other domain.

The Open/Closed Principle (OCP) demands that modules be extensible without modifying their source code. Plugin architectures, strategy patterns, and abstract base classes with concrete subclasses all embody OCP: adding a new payment processor should involve creating a new class that implements a PaymentProvider interface, not editing a switch statement inside an existing OrderProcessor. This is where SOLID intersects with DRY -- extending behavior through new code rather than modifying existing code reduces the risk that a change in one area breaks another.

Liskov Substitution (LSP) requires that any subtype be usable wherever its base type is expected without altering the correctness of the program. A Square class that inherits from Rectangle but enforces equal sides violates LSP because code expecting a Rectangle may set width and height independently, causing surprising behavior. Violations of LSP force callers to inspect the concrete type at runtime, which defeats polymorphism and scatters type-checking logic throughout the codebase.

Interface Segregation (ISP) and Dependency Inversion (DIP) complete the set. ISP pushes designers toward narrow, purpose-specific interfaces: a fat API that exposes CRUD methods, reporting methods, and audit methods forces every client to depend on functionality it does not use, coupling it to changes in unrelated features. DIP mandates that high-level policy modules depend on abstractions rather than concrete infrastructure, enabling the policy to be tested in isolation and deployed with different infrastructure implementations.

## Practical Guidance
- Apply SOLID incrementally: refactor toward the principles when you encounter pain -- rigidity, fragility, or difficult testing -- rather than designing an entire system around them upfront.
- Use automated tests as a safety net when restructuring code to satisfy SOLID; the principles and TDD reinforce each other.
- When a class exceeds roughly 300 lines or has more than seven dependencies, examine it for SRP violations and consider extraction.

## Common Misreadings
The most pervasive misreading treats SOLID as a set of rules that must be applied universally and uniformly, leading to over-abstracted codebases where every class is an interface with a single implementation. This creates indirection without benefit and violates KISS. Another error is interpreting SRP to mean that every class should have exactly one method; responsibility refers to an axis of change driven by a business actor or requirement, not to a method count. A third misreading applies SOLID only at the class level while ignoring package and module boundaries; the principles scale up to component and service architecture when applied thoughtfully.

## Interactions
SOLID and DRY are mutually reinforcing: single-responsibility classes naturally centralize knowledge about one concern, and open/closed extension avoids duplicating modification logic. The Law of Demeter implements ISP and DIP at the call-site level by restricting how far an object graph can be traversed, which keeps interfaces narrow and dependencies abstract. KISS acts as a counterbalance: when applying a SOLID principle would add more complexity than it removes, simplicity should win. The Principle of Least Astonishment benefits from SOLID because well-structured, substitutable components behave predictably -- a method called on a base type behaves correctly regardless of which subtype is actually present.

---

*Based on: Martin, "Agile Software Development" (2002)*
