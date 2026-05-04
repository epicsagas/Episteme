# Lehman's Laws

## Statement
Lehman's Laws are a set of empirically derived principles describing how large software systems evolve: they must change continually to remain useful, they grow more complex over time unless actively simplified, and the effort required to maintain them increases at a predictable rate.

## Origin
Meir M. Lehman formulated these laws through longitudinal studies of large-scale software systems, beginning with his observations of IBM's OS/360 in the 1970s and culminating in his 1980 paper "Programs, Life Cycles, and Laws of Software Evolution." Lehman classified software into three categories: S-type (specifiable, mathematical), P-type (defined by the problems they solve), and E-type (embedded in and must reflect a changing real world). The laws apply to E-type systems, which constitute the majority of production software.

## Software Implications
Law 1, Continuing Change, states that an E-type system must be continually adapted or it becomes progressively less satisfactory. This explains why successful products never reach a "done" state: the business environment they serve keeps evolving, and the software must evolve with it. A system that was perfectly suited to its domain at release becomes obsolete within months if not updated.

Law 2, Increasing Complexity, observes that as a system evolves, its complexity increases unless active maintenance reduces it. Without deliberate simplification, features accumulate in layers, interactions multiply, and the system becomes harder to reason about. This is the thermodynamic arrow of entropy applied to code: complexity grows by default, and reducing it requires explicit energy investment.

Law 6, Continuing Growth, and Law 7, Declining Quality, together explain why long-lived systems eventually face a crisis point. The system must grow to remain relevant, but growth without architectural investment degrades quality, which slows growth, creating a feedback loop. Organizations that recognize this pattern invest proactively in modularization, abstraction boundaries, and periodic restructuring to counteract the natural trend.

The remaining laws address organizational dynamics: development pace self-regulates around a stable mean (Law 4), incremental change has sustainable size limits (Law 5), and evolution must be driven by feedback from operational use (Law 8).

## Practical Guidance
- Budget maintenance effort as a permanent, growing allocation rather than a one-time project.
- Track architectural complexity metrics such as coupling and cyclomatic complexity trends over releases.
- Invest in modular architecture early; boundaries slow the rate at which complexity propagates across modules.
- Collect and act on operational telemetry continuously to drive evolution decisions with real usage data.

## Common Misreadings
Some interpret Lehman's Laws as a deterministic prediction that all software inevitably becomes unmaintainable, which is fatalistic and inaccurate. The laws describe what happens without intervention; active simplification and architectural investment can sustain system health indefinitely. Another misreading applies the laws to small, short-lived projects where the evolutionary dynamics do not have time to manifest. A third error treats the laws as prescriptive guidelines rather than descriptive patterns; they describe what tends to happen, not what should happen.

## Interactions
Lehman's Laws provide the macro-level framework within which Technical Debt accumulates: debt is the mechanism by which increasing complexity manifests as economic cost. The Boy Scout Rule is a micro-level countermeasure against Law 2's complexity growth. The Broken Windows Theory explains the social dynamic that accelerates the quality decline predicted in Law 7. The Testing Pyramid must scale with system growth to maintain defect detection, addressing the combined pressure of Laws 2 and 6. Sturgeon's Law applies to the output of evolving systems: as complexity grows, the proportion of high-quality modules tends to shrink without deliberate effort.

---

*Based on: Lehman, "Laws of Software Evolution" (1980)*
