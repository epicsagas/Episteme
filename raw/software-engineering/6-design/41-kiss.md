# KISS

## Statement
KISS -- Keep It Simple, Stupid -- directs engineers to prefer the simplest solution that correctly satisfies requirements over clever or elaborate alternatives. Simplicity reduces the surface area for defects, lowers the cognitive burden on readers, and makes future modification straightforward. The principle applies at every level: individual functions, module interfaces, system architectures, and deployment pipelines all benefit from minimizing unnecessary complexity.

## Origin
The acronym originated at Lockheed in the 1960s, attributed to aircraft engineer Kelly Johnson, who challenged his design teams to build jet aircraft simple enough that an average mechanic in the field could repair them with limited tools under combat conditions. The aerospace community adopted the phrase widely, and software engineering inherited it as a counterweight to the profession's natural tendency toward over-engineering. While no single software publication introduced KISS, it appears consistently in programming culture from the Unix philosophy's "do one thing well" through modern agile methodology's emphasis on working software over comprehensive documentation.

## Software Implications
Cognitive load is the hidden cost that KISS targets. A developer reading a recursive, higher-order-function chain that cleverly compresses ten lines into three must hold multiple abstractions in working memory simultaneously, which slows comprehension and increases the chance of introducing bugs during modification. A straightforward loop with named variables communicates intent immediately and leaves less room for misinterpretation. The same dynamic plays out at the architectural level: a microservices topology with seventeen specialized services communicating via event sourcing may be intellectually satisfying to design, but a monolithic application with three well-bounded modules is often simpler to deploy, debug, and evolve for teams that have not yet reached the scale that justifies distribution.

KISS does not advocate avoiding necessary complexity. A real-time bidding system handling millions of events per second requires sophisticated engineering -- message queues, caching layers, careful concurrency control. The principle argues instead that each piece of complexity should earn its place by solving a problem that exists today, not one that might hypothetically arise next quarter. Premature optimization, speculative generalization, and "we might need this later" abstractions are the most common violations of KISS in production codebases.

Testing provides a natural barometer for simplicity. Code that is difficult to unit-test often carries unnecessary complexity: hidden dependencies, global state, or tangled control flow. Refactoring for testability frequently simplifies the design as a side effect, which is why KISS and test-driven development reinforce each other. Simple code is testable code, and testable code tends to stay simple.

## Practical Guidance
- Before adding an abstraction layer, write the straightforward version first; extract patterns only after repetition confirms the need.
- During code review, challenge any indirection that requires more than one sentence of explanation to justify its existence.
- When a module grows complex enough to need a README or walkthrough, consider whether splitting it into smaller, simpler pieces would eliminate the need for the documentation itself.

## Common Misreadings
One frequent misreading equates simplicity with naivety: choosing an O(n-squared) algorithm because it is shorter to write, even though a straightforward O(n log n) alternative exists. Simplicity refers to the cognitive complexity of the solution, not its brevity or its ignorance of performance characteristics. Another mistake is using KISS to resist all architectural evolution; a startup that never graduates from its initial single-server deployment because "it works fine" may be conflating simplicity with stagnation. KISS argues against unnecessary complexity, not against complexity that genuine scale or genuine requirements demand.

## Interactions
KISS and YAGNI (You Aren't Gonna Need It) are kindred principles: both reject speculative design decisions that add complexity without current value. The DRY principle can conflict with KISS when aggressive deduplication produces abstractions that are harder to understand than the duplicated code would have been; the right balance depends on whether the duplication represents the same knowledge or merely similar syntax. SOLID principles generally support KISS by promoting cohesive, focused modules that are individually simple even when the overall system is rich. The Principle of Least Astonishment aligns with KISS because simple behavior is predictable behavior, and predictable behavior does not surprise users.

---

*Based on: Johnson, Lockheed (1960s)*
