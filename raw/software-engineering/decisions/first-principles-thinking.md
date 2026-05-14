# First Principles Thinking

## Statement
First Principles Thinking is a reasoning method that decomposes a problem into its most fundamental, irreducible truths and then reconstructs a solution upward from those foundations rather than reasoning by analogy to existing solutions. It replaces the question "what do others do?" with "what do we know to be true at the most basic level?"

## Origin
The approach traces to Aristotle, who defined a first principle as "the first basis from which a thing is known" in his Physics and Metaphysics. The method became central to scientific inquiry through thinkers such as Descartes, who used radical doubt to strip away assumptions, and Euclid, whose geometric system was built from axioms. In the modern era, the approach has been championed by entrepreneurs and engineers, most visibly by Elon Musk in the context of aerospace and automotive design.

## Software Implications
When a team encounters a performance bottleneck, analogy-based thinking might suggest adding caching because that is what similar systems have done. First Principles Thinking instead decomposes the problem into latency components: network round-trip time, serialization cost, query execution time, and lock contention. If the dominant component is lock contention, caching the result set will not help, and the first-principles analysis will direct attention to the actual constraint.

In system design, the method prevents cargo-cult architecture. A team building a new service might reflexively adopt the microservices pattern because the industry conversation is dominated by it. A first-principles analysis would start by identifying the actual deployment independence, scaling, and team-autonomy requirements, and only then determine whether a monolith, a modular monolith, or a distributed architecture best satisfies them.

First principles also apply to estimation and planning. Rather than estimating a project by analogy to a previous project that "felt similar," a first-principles decomposition breaks the work into atomic tasks whose effort can be reasoned about individually. The resulting estimate is grounded in the actual work rather than in a fuzzy memory of a different project.

## Practical Guidance
- When facing a design decision, list every assumption you are carrying, then challenge each one by asking "how do I know this is true?"
- Decompose the problem until you reach constraints governed by physics, mathematics, or established law -- those are your first principles.
- Rebuild the solution from those constraints upward, checking at each layer that the reasoning is sound before adding the next.

## Common Misreadings
A frequent error conflates first principles with ignoring all prior art. The method does not require pretending that existing solutions do not exist; it requires understanding why they exist and verifying that their assumptions still hold. Another misreading treats every personal intuition as a "first principle." A true first principle is independently verifiable and non-controversial within the relevant domain, not simply a strongly held opinion.

## Interactions
First Principles Thinking is the natural antidote to the Hype Cycle because it grounds technology decisions in verifiable constraints rather than social proof. It complements Inversion: while inversion asks "what would guarantee failure?" to reveal hidden risks, first principles asks "what is undeniably true?" to expose hidden opportunities. The Map Is Not the Territory provides a philosophical foundation, since first principles thinking is essentially the practice of testing your maps against the territory rather than treating the maps as authoritative. Occam's Razor reinforces the decomposition step by favoring the smallest set of foundational truths.

---

*Based on: Aristotle, Physics and Metaphysics (c. 350 BCE)*
