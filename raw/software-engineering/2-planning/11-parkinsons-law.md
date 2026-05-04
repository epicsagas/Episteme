# Parkinson's Law

## Statement
Parkinson's Law observes that work expands to fill the time allocated for its completion. Give a team two weeks to deliver a feature, and it will take roughly two weeks; give the same team four weeks for the same feature, and it will take roughly four weeks. The deadline, not the task's inherent difficulty, often determines the duration.

## Origin
Cyril Northcote Parkinson, a British naval historian and public administrator, published this observation in a 1955 essay in The Economist, later expanding it into the 1957 book "Parkinson's Law: The Pursuit of Progress." Parkinson drew on his experience in the British Civil Service, where he noted that bureaucracies grow even as the amount of actual work remains constant. Officials create work for one another through committee meetings, memo writing, and procedural compliance, ensuring that every available hour is consumed regardless of the underlying task's requirements.

## Software Implications
The law operates visibly in sprint planning. A team that commits to delivering three features in a two-week sprint will typically deliver those features by the end of the sprint. The same team given a four-week sprint for the same three features will find ways to use the extra time: additional refactoring, exploratory spikes, broader testing, and extended code review cycles. Some of this expanded work is genuinely valuable, but much of it represents gold-plating that would not occur under tighter constraints.

Meetings are another clear example. A standing meeting scheduled for one hour will consume one hour regardless of whether the agenda requires 15 minutes or 60. Participants adapt their pace and depth of discussion to fill the available slot. Teams that switch to 25-minute meetings often discover that they accomplish the same outcomes in less time because the tighter constraint forces prioritization and focus.

The inverse is equally important: artificially compressed timelines can produce better outcomes by forcing ruthless prioritization. A team that must deliver a demo in one week will cut scope aggressively, focus on the most visible features, and defer nonessential work. The result is often more aligned with actual business priorities than what the team would have produced given a leisurely timeline.

## Practical Guidance
- Use time-boxed sprints or iterations to impose natural deadlines that prevent scope expansion.
- Set aggressive but achievable deadlines to force prioritization and limit gold-plating.
- Shorten recurring meetings to the minimum duration that allows the agenda to be covered and observe whether outcomes degrade.
- Distinguish between productive work that fills available time and artificial work that exists only to justify the timeline.

## Common Misreadings
Parkinson's Law does not justify setting impossible deadlines. Work expands to fill available time, but it cannot be compressed below the minimum effort required for correctness and quality. A one-week deadline for a feature that genuinely requires three weeks produces burnout, technical debt, and defects, not efficiency.

The law also does not imply that all expanded work is waste. Some of the additional activity that fills a generous timeline, such as thorough testing, documentation, and architectural consideration, is valuable. The insight is that without a constraining deadline, the boundary between valuable and wasteful expansion blurs.

## Interactions
Parkinson's Law amplifies Hofstadter's Law, because even after planners account for underestimation, the remaining time will be consumed by expanded scope. It interacts with the Ninety-Ninety Rule: the "second 90 percent" of a project expands to fill whatever schedule buffer exists. Goodhart's Law operates when teams use story points or velocity as the measure that becomes a target, inflating estimates to ensure work fills the sprint. The Ringelmann Effect compounds Parkinson's Law because larger teams generate more internal coordination work, which conveniently fills any available slack in the schedule.

---

*Based on: Parkinson, The Economist (1955)*
