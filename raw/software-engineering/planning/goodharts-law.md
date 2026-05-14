# Goodhart's Law

## Statement
Goodhart's Law warns that when a metric is adopted as a target for decision-making, it ceases to function as a reliable measure of the underlying quality it was intended to track. The act of optimizing for the metric distorts the behavior it captures, separating the proxy from the reality it represents.

## Origin
Charles Goodhart, an economist and advisor to the Bank of England, articulated this principle in the context of monetary policy during the 1970s. Goodhart observed that when central banks used monetary aggregates as targets for interest rate policy, the relationship between those aggregates and actual economic conditions broke down. The insight was later generalized by Marilyn Strathen and others to apply beyond economics to any domain where quantitative measures influence behavior. The formulation "when a measure becomes a target, it ceases to be a good measure" has become the standard summary.

## Software Implications
Software organizations routinely encounter Goodhart's Law when they adopt metrics to drive engineering behavior. Code coverage is the canonical example. When a team measures code coverage to understand test thoroughness, the metric provides useful diagnostic information. When management sets a target of 80 percent coverage and ties performance reviews to hitting that target, developers respond by writing tests that maximize coverage with minimal effort: tests that exercise code paths without asserting meaningful behavior, tests that mock every dependency into trivial compliance, and tests that skip edge cases because those do not increase the coverage percentage. Coverage rises, but defect rates remain unchanged.

Velocity in Agile planning suffers a similar corruption. Story points are useful as a planning tool for forecasting how much work a team can complete in a sprint. When velocity becomes a target, teams inflate estimates to ensure they consistently "hit their number," and the metric loses its calibration to actual throughput. Stakeholders who compare velocity across teams compound the damage, because each team has its own point scale and the numbers are not comparable.

The law applies to individual metrics as well. Lines of code, number of commits, number of bugs resolved, and number of pull requests merged all become gameable once they are tied to rewards or penalties. The gaming is usually not malicious; people respond rationally to the incentives they are given. The problem is that the incentive is misaligned with the desired outcome.

## Practical Guidance
- Use metrics as diagnostic instruments, not as targets; distinguish between observing a trend and optimizing for a number.
- When you must set quantitative goals, use a balanced set of metrics that span quality, speed, and outcomes so that gaming one metric degrades another.
- Periodically audit whether your metrics still correlate with the qualities you care about, and replace them when the correlation breaks down.
- Keep the primary reward structure tied to qualitative outcomes, such as customer satisfaction and production reliability, rather than to internal process metrics.

## Common Misreadings
Goodhart's Law does not argue against measurement. Gilb's Law makes the complementary case: imperfect measurement beats no measurement. Goodhart's Law warns specifically about the transition from measurement to target, not about the act of measuring itself. A team that monitors code coverage without setting a target can use the data to identify untested modules without triggering the gaming response.

Another misreading is the assumption that all targets are harmful. Targets can be effective when they are aligned with the actual outcome, difficult to game, and used as one input among many rather than as a sole criterion. The law cautions against naive target-setting, not against goal-setting in general.

## Interactions
Goodhart's Law is the planning counterpart to Premature Optimization: both describe the damage caused by optimizing for a proxy rather than for the real goal. It intensifies Parkinson's Law because teams that must hit a velocity target will expand work to fill the sprint rather than delivering early. It interacts with the Peter Principle when managers who lack technical depth rely on metrics as a substitute for judgment, because they cannot evaluate the work directly. Gilb's Law provides the foundation that Goodhart's Law builds upon: measurement is valuable, but measurement-as-target is dangerous.

---

*Based on: Goodhart, "Problems of Monetary Management: The UK Experience" (Reserve Bank of Australia, 1975)*
