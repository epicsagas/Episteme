# DRY

## Statement
DRY -- Don't Repeat Yourself -- mandates that every piece of domain knowledge must have exactly one authoritative representation within a system. When the same rule, constant, algorithm, or configuration appears in multiple locations, any future change must be applied consistently to every copy, and a single missed update introduces subtle defects. The principle extends beyond source code to database schemas, build configurations, documentation, and test fixtures.

## Origin
Andrew Hunt and David Thomas coined the term in their 1999 book "The Pragmatic Programmer," drawing on earlier ideas about single source of truth that existed in relational database theory and structured programming. They framed DRY not merely as avoiding copy-pasted code but as a broader discipline of knowledge centralization, stating: "Every piece of knowledge must have a single, unambiguous, authoritative representation within a system" (Hunt & Thomas, 1999, p. 27). The concept resonated immediately with the software community and became one of the most widely cited heuristics in programming.

## Software Implications
Violations of DRY manifest as scattered duplication that erodes maintainability. An email validation regex copied across twelve microservices drifts over time: one service rejects valid addresses, another allows malformed ones, and no one knows which copy is canonical. Extracting that regex into a shared library with a single versioned artifact eliminates the inconsistency and ensures that a bug fix propagates everywhere in one deployment.

DRY also shapes data modeling. A customer address stored in both the orders table and the billing table creates an update anomaly: when a customer moves, one table gets corrected while the other retains stale data. Normalization -- the relational-database analog of DRY -- eliminates this by storing the address once and referencing it via foreign key. The same thinking applies to configuration: environment variables, feature flags, and service endpoints should be defined in one place and injected everywhere else, not duplicated in Dockerfiles, Terraform modules, and application YAML.

However, DRY can be over-applied. Premature abstraction -- extracting shared code before the common pattern is well understood -- couples modules that should evolve independently. Three similar-looking validation functions might serve different business contexts that will diverge over time; merging them into one creates a brittle abstraction that satisfies DRY but violates the Single Responsibility Principle and the Open/Closed Principle from SOLID. Wise engineers wait until a pattern repeats with clear convergence before centralizing it.

## Practical Guidance
- When you find yourself copying code, pause and ask whether the two copies serve the same business concept or merely look similar by coincidence.
- Centralize configuration, constants, and domain rules into single modules or services; consume them by reference, not by duplication.
- Prefer composition over inheritance when eliminating duplication; shared behavior via mixins or traits often preserves independence better than a deep class hierarchy.

## Common Misreadings
The most harmful misreading treats DRY as a mandate to eliminate all visual similarity in code, even when the duplicated fragments serve different domain purposes. Two identical-looking string-formatting blocks in separate modules are not necessarily the same knowledge -- they may represent different business rules that happen to share syntax today but will diverge tomorrow. Another common mistake is applying DRY across system boundaries: sharing a database between microservices to avoid duplicating data couples those services tightly and undermines the autonomy that the microservice architecture was meant to provide.

## Interactions
DRY is closely related to the Single Responsibility Principle in SOLID: when a piece of knowledge has one authoritative location, that location has one clear reason to change. The Law of Demeter supports DRY by discouraging deep object traversals that implicitly duplicate knowledge of an object's internal structure across call sites. KISS reinforces DRY from a different angle: unnecessary repetition adds complexity, but so does over-abstracted code that tries to be DRY where duplication would be simpler. The Principle of Least Astonishment benefits from DRY because consistent behavior follows naturally when logic lives in one place rather than being reimplemented with subtle variations.

---

*Based on: Hunt & Thomas, "The Pragmatic Programmer" (1999)*
