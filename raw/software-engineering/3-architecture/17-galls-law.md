# Gall's Law

## Statement
Gall's Law holds that every complex system that functions reliably in the real world evolved incrementally from a simpler system that also functioned reliably. A brand-new complex system, designed from scratch without a working predecessor, will not work. The law implies that complexity must be earned through iterative growth, not invented wholesale, because only operating experience reveals which parts of a design actually matter.

## Origin
John Gall, an American pediatrician and systems theorist, introduced this principle in his 1975 book "Systemantics: How Systems Really Work and How They Fail." Written with wry humor, the book catalogs recurring failure modes in organizations and technical systems. Gall's central observation was that the systems people depend on — hospitals, airlines, software — share a common developmental pattern: they started small, worked, and grew more complex over time in response to real demands rather than speculative ones.

## Software Implications
The law directly challenges the instinct to greenfield a sophisticated architecture before a single user has touched the product. A startup that sketches out a microservices mesh with event sourcing, CQRS, and Kubernetes orchestration on day one is designing a complex system without a working simple ancestor. The result is typically months of infrastructure work before any user value ships, and the architecture reflects assumptions rather than observed bottlenecks.

Gall's Law validates patterns like starting with a monolith and extracting services only when throughput or team-scaling demands it. It also explains why rewrites so often fail: the rewritten system lacks the years of edge-case fixes, operational adjustments, and implicit knowledge baked into the original. Amazon, Shopify, and GitHub all famously evolved from simple monolithic deployments toward distributed architectures only after their simpler systems had proven what actually needed scaling.

The law applies at smaller scales too. A well-factored module begins as a straightforward implementation, accrues abstractions as requirements diversify, and only then introduces generality. Attempting to build the fully generalized version first leads to speculative abstractions that fit no real use case well.

## Practical Guidance
- Ship the simplest system that solves today's confirmed problem, then add complexity in response to evidence from production.
- Treat architectural diagrams for future scale as hypotheses, not blueprints; validate them against real load and real users before committing.
- When rewriting, preserve the domain knowledge embedded in the old system by studying its runtime behavior, not just its source code.
- Resist the urge to generalize from a single use case; wait until two or three concrete scenarios demand shared abstraction.

## Common Misreadings
Some interpret Gall's Law as a prohibition against any upfront design, using it to justify shipping messy prototypes and hoping evolution will fix them. The law says the system must work at every stage of growth — a broken prototype does not satisfy the "simple system that works" requirement. Others misapply the law to argue against all planning, when in fact Gall advocates for evolutionary growth guided by feedback, not unplanned chaos.

## Interactions
Gall's Law provides the evolutionary mechanism that prevents the Second System Effect from taking hold: if you build a new system by extending a proven simple one rather than inventing from scratch, the temptation to overdesign is restrained by the existing working architecture. It tempers the scope expansion described by Zawinski's Law by anchoring growth in real demand rather than speculative feature lists. The Law of Leaky Abstractions is easier to manage under Gall's Law because incremental growth exposes leaks one at a time instead of confronting designers with all of them at once. Hyrum's Law interacts constructively too: evolving a system preserves the observable behaviors that consumers depend on, whereas a clean-room rewrite discards them.

---

*Based on: Gall, "Systemantics" (1975)*
