# Inversion

## Statement
Inversion is a problem-solving discipline that works backward from an undesired outcome to identify the actions, conditions, and omissions that would produce it, then avoids those causes. Rather than asking "how do we succeed?" it asks "how would we guarantee failure?" and uses the answer as a roadmap for what to prevent.

## Origin
The mathematician Carl Gustav Jacob Jacobi famously advised his students to "invert, always invert" when confronted with difficult problems, observing that many equations yield more readily when the unknowns are rearranged. Charlie Munger, vice-chairman of Berkshire Hathaway, popularized Jacobi's insight as a general thinking tool, arguing that many complex problems are easier to solve backward than forward. The approach has roots in stoic philosophy, where premeditatio malorum -- the premeditation of evils -- was practiced as a method for building resilience and clarifying priorities.

## Software Implications
Pre-mortems are inversion applied to project planning. Before work begins, the team imagines that the project has already failed catastrophically and each member writes down the most plausible cause. The resulting list is a prioritized risk register generated in minutes rather than months. This exercise surfaces dependencies, unclear requirements, and single points of failure that forward-looking optimism typically obscures.

In security engineering, inversion is the default posture. Threat modeling does not ask "how do we build a secure system?" -- it asks "how would an attacker compromise this system?" By enumerating attack paths in reverse, security teams identify the most damaging entry points and allocate hardening effort where it matters most.

Inversion also clarifies architectural decisions. Instead of listing the qualities a system should have, an inverted analysis lists the qualities that would make the system unworkable: tight coupling to a specific vendor, deployment procedures that require downtime, insufficient observability to detect degradation. Avoiding these anti-qualities often produces a more focused design than chasing an abstract list of positive attributes.

## Practical Guidance
- Run a pre-mortem at the start of every significant initiative: assume failure, enumerate causes, then build mitigation plans for each.
- When designing a system, list the top five ways it could become unmaintainable and ensure the architecture explicitly prevents each one.
- In code review, ask not only "does this work?" but "what input or condition would make this fail catastrophically?"

## Common Misreadings
Inversion is sometimes mistaken for pessimism or negativity. The method does not dwell on failure for its own sake; it uses the imagined failure as a diagnostic tool that improves the probability of success. Another error is to stop at identifying failure modes without acting on them. Listing risks without building mitigations provides a false sense of thoroughness while delivering none of the benefit.

## Interactions
Inversion pairs naturally with First Principles Thinking: first principles identifies the foundational truths to build on, while inversion identifies the foundational errors to avoid. It directly counters the Sunk Cost Fallacy because the inverted question "if we had not already invested in this, would we start?" strips emotional attachment from the evaluation. Confirmation Bias is also weakened by inversion, since actively searching for ways the plan could fail forces engagement with disconfirming evidence. The Pareto Principle helps prioritize inverted findings: the few failure modes with the highest impact should receive the most defensive effort.

---

*Based on: Jacobi; Munger popularization*
