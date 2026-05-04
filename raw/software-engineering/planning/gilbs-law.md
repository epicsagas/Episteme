# Gilb's Law

## Statement
Gilb's Law asserts that anything worth understanding can be measured, and that imperfect measurement reliably produces better decisions than unaided intuition. The emphasis is not on precision but on establishing a quantitative baseline that enables observation of trends, comparison of alternatives, and detection of regressions.

## Origin
Tom Gilb, a software engineering consultant and metrics advocate, advanced this principle in his 1988 book "Principles of Software Engineering Management." Gilb argued against the common objection that software quality attributes such as reliability, usability, and maintainability are too subjective to measure. His position was that any attribute important enough to influence design decisions is important enough to quantify, even if the quantification is rough. The law provides the philosophical foundation for evidence-based software management.

## Software Implications
The law operates as a counterweight to the instinct to avoid measurement because it cannot be done perfectly. A team that refuses to track deployment frequency because the definition of a "deployment" is ambiguous loses the ability to observe whether their CI/CD pipeline improvements are having any effect. A team that counts deployments, even with an imperfect definition, can track trends over time and compare periods before and after a process change. The imperfect number enables learning; the absence of a number prevents it.

Performance optimization illustrates the principle directly. An engineer who says "the API feels slow" has no basis for deciding whether to invest effort in optimization or where to focus that effort. An engineer who measures response latency at the 95th percentile over a 24-hour window has a concrete baseline, can identify which endpoints are outliers, and can verify after optimization whether the change produced the desired improvement. The measurement may not capture every dimension of performance, but it enables targeted action where intuition alone cannot.

The law also applies to team dynamics. A manager who tracks cycle time, even approximately, can observe whether a process change such as adopting trunk-based development or reducing work-in-progress limits has the intended effect. Without the measurement, the manager relies on subjective impressions that are biased toward whichever outcome they expected.

Gilb's Law is not a license to measure everything. Measurement carries a cost in data collection, analysis, and attention. The law argues that when a question matters enough to drive decisions, the cost of imperfect measurement is lower than the cost of deciding blindly.

## Practical Guidance
- When a design or process decision is being debated, identify what observable quantity would settle the argument and start measuring it immediately, even with a crude instrument.
- Treat initial measurements as approximations that will improve over time; do not delay action while designing a perfect measurement framework.
- Track metrics over time rather than relying on snapshots, because trends reveal more than isolated values.
- Review periodically whether your current metrics still inform the decisions you are making, and discard ones that have become noise.

## Common Misreadings
Gilb's Law is sometimes cited as an argument for comprehensive metrics dashboards that track dozens of signals simultaneously. The law advocates for purposeful measurement tied to specific decisions, not for surveillance. Measuring everything is as unhelpful as measuring nothing, because the signal drowns in noise.

Another misreading equates Gilb's Law with the claim that everything can be reduced to numbers. Some qualities resist direct quantification, such as team morale or code elegance. Gilb's response would be that proxy measures, such as voluntary attrition rate or code review turnaround time, still provide more actionable information than gut feeling.

## Interactions
Gilb's Law provides the measurement foundation that Goodhart's Law warns about: without Gilb's commitment to measurement, Goodhart's caution about targets is moot. The two laws form a pair: measure everything that matters (Gilb), but do not let the measurement become the goal (Goodhart). Premature Optimization is the practice that Gilb's Law corrects: instead of guessing where performance bottlenecks lie, measure and then optimize the code paths that profiling identifies. Parkinson's Law can be partially counteracted by measuring actual versus estimated effort, providing the empirical feedback that Hofstadter's Law says planners always underestimate.

---

*Based on: Gilb, software engineering metrics*
