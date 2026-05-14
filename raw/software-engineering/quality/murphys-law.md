# Murphy's Law

## Statement
Murphy's Law asserts that any failure mode possible within a system will eventually manifest under real-world conditions. The principle compels engineers to design for the worst case rather than the happy path.

## Origin
Edward A. Murphy Jr. articulated the adage during aerospace engineering experiments at Edwards Air Force Base in 1949, after a technician wired sensors incorrectly in every possible wrong orientation. The phrase entered popular culture as a general observation about entropy and human error. In software engineering, it functions as a design heuristic: assume every external dependency will fail, every input will be malformed, and every concurrent process will race.

## Software Implications
A service that calls a third-party API without a timeout will eventually hang indefinitely when that API's server stalls. A form validator that assumes numeric fields contain only digits will break the moment a user pastes text from a spreadsheet. A distributed system without retry logic collapses when a single network packet drops. Murphy's Law transforms these from hypothetical concerns into design requirements.

Defensive programming is the direct engineering response: validate all inputs at system boundaries, set timeouts on every network call, implement circuit breakers around external dependencies, and write idempotent handlers so duplicate messages cause no harm. Chaos engineering operationalizes the law by deliberately injecting failures in production to verify that defenses actually work. Teams that internalize Murphy's Law write failure-mode-and-effects analyses before architecture reviews and treat production incidents as specification gaps rather than surprises.

The law also justifies comprehensive logging and observability. When something goes wrong, detailed traces are the only way to reconstruct what happened and confirm the fix addresses the root cause rather than a symptom.

## Practical Guidance
- Add timeouts and retry budgets to every outbound network call.
- Validate and sanitize all inputs at module boundaries, not just at the UI layer.
- Write a post-mortem for every production incident and extract a concrete defensive check from each.
- Run fault-injection tests in CI to prove that fallback paths execute correctly.

## Common Misreadings
Engineers sometimes invoke Murphy's Law to justify over-engineering every component for every conceivable failure, which leads to excessive abstraction and delayed delivery. The productive reading is proportional defense: invest resilience in proportion to the blast radius of the failure, not in proportion to how frightening the failure seems. Another misunderstanding is treating the law as fatalism rather than motivation; the goal is not to prevent all failures but to ensure that when they occur, the system degrades gracefully and recovers quickly.

## Interactions
Murphy's Law underpins the entire discipline of defensive programming and pairs naturally with Postel's Law, which mandates lenient input handling. It amplifies the need for the Testing Pyramid by demanding thorough edge-case coverage at the unit level. The law intersects with Technical Debt because shortcuts that ignore failure modes become debt that materializes as production incidents. Lehman's Laws reinforce the urgency: as systems evolve and grow more complex, the surface area for Murphy-style failures expands.

---

*Based on: Bloch, "Murphy's Law and Other Reasons Why Things Go Wrong" (Price/Stern/Sloan, 1977)*
