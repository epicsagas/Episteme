# YAGNI

## Statement
YAGNI ("You Aren't Gonna Need It") prohibits implementing functionality before a concrete requirement demands it. The principle holds that speculative features waste development effort, introduce unnecessary complexity, and often solve the wrong problem.

## Origin
The acronym emerged from the Extreme Programming community in the late 1990s, with Ron Jeffries and Kent Beck as primary advocates. The principle appeared as one of the core XP design practices: build only what is needed today, design for today's requirements, and refactor when new requirements emerge. The economic logic is straightforward: effort spent on unused features is effort not spent on features users actually need, and the future requirements that justified the speculation frequently never materialize.

## Software Implications
A developer builds a generic plugin architecture for a feature that currently has one implementation, anticipating that future features will plug into the same framework. Six months later, no second plugin has materialized, but the generic architecture has added three layers of indirection that every developer must navigate when modifying the original feature. The upfront investment not only failed to pay off but actively increased the cost of ongoing maintenance. YAGNI predicts this outcome: until the second plugin requirement is real, the abstraction is speculative waste.

The principle applies at every scale, from individual functions to system architecture. At the function level, adding configuration parameters that no caller currently uses is a YAGNI violation. At the architecture level, introducing a message queue for decoupling before any second consumer exists is the same pattern. In each case, the cost is not just the initial implementation time but the ongoing cognitive overhead of understanding and maintaining machinery that serves no present purpose.

YAGNI does not prohibit all forward-looking design. A team that knows a database migration is planned for next quarter should choose an abstraction that accommodates it, provided the accommodation costs no more than the simple solution. The threshold is whether the design decision adds present cost for a hypothetical future benefit. If the generalization is free or cheaper than the narrow solution, YAGNI does not apply. The discipline is distinguishing genuine economies from speculative over-engineering.

## Practical Guidance
- When tempted to add a feature "just in case," write the use case as a concrete user story; if no real user can be identified, defer the feature.
- Prefer composition over anticipation: build narrow, focused components that can be combined later rather than monolithic components with configuration knobs for every imagined scenario.
- Refactor aggressively when real requirements do arrive; the cost of targeted refactoring is almost always lower than the cost of maintaining unused generality.
- Track the ratio of used to unused configuration options in your system as a YAGNI health metric.

## Common Misreadings
The most common misreading interprets YAGNI as a prohibition against any design thinking, which produces code that is brittle and difficult to extend when real requirements arrive. YAGNI prohibits speculative implementation, not thoughtful design; a well-factored narrow solution is easier to extend than a poorly factored one. Another error is conflating YAGNI with never writing abstractions; the principle targets premature abstraction, not all abstraction. A third misreading uses YAGNI to justify skipping infrastructure that supports future development velocity, such as CI pipelines or logging frameworks, conflating speculative features with foundational engineering practices.

## Interactions
YAGNI directly reduces Technical Debt by preventing speculative features that become unmaintained baggage. It complements the Boy Scout Rule: building only what is needed produces simpler code that is easier for the next developer to improve. The principle aligns with Sturgeon's Law because speculative features, constituting work done without real requirements, almost always fall into the mediocre majority. Kernighan's Law reinforces YAGNI: speculative code that is never exercised is code that will be impossible to debug when a requirement eventually does materialize, because no one understands what it was supposed to do.

---

*Based on: Jeffries/Beck, Extreme Programming (1999)*
