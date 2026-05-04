# Technical Debt

## Statement
Technical Debt is the accumulated cost of choosing expedient solutions over thorough ones, payable as increased effort for every future change to the affected code. Like financial debt, it compounds: unpaid principal generates interest in the form of slower development velocity.

## Origin
Ward Cunningham coined the metaphor at the OOPSLA conference in 1992 while presenting the WyCash Portfolio Management System. He used the financial debt analogy to explain to non-technical stakeholders why the team needed to allocate time for restructuring. The metaphor proved powerful because it translated an abstract code quality concern into a concrete economic concept that business leaders already understood: borrow now, pay later, with interest.

## Software Implications
A startup ships a hardcoded configuration to meet a launch deadline, creating deliberate technical debt with a known repayment date. The debt becomes toxic if the team never schedules the refactor: each subsequent feature that touches the hardcoded value takes longer because developers must trace through scattered magic strings. After several months, the interest exceeds the original principal, and the cost of a rewrite rivals the cost of the entire initial feature.

Technical debt is not always deliberate. Accidental debt accumulates when developers make suboptimal choices without realizing it, often because domain understanding evolves after the code is written. A data model designed before the business rules were clear becomes a poor fit once requirements solidify. Unlike deliberate debt, accidental debt carries no payoff analysis because the cost was never consciously accepted.

The key discipline is making debt visible. Teams that maintain a prioritized debt register, tagged with the interest rate (development friction) and principal (effort to fix), can make rational decisions about repayment. Without visibility, debt accumulates silently until a team finds itself unable to estimate feature work reliably because every change requires untangling coupled modules first.

## Practical Guidance
- Tag every shortcut with a comment linking to a tracked debt item that estimates repayment cost and current friction.
- Allocate a fixed percentage of each sprint capacity to debt repayment; ten to twenty percent is a common baseline.
- Classify debt as deliberate, accidental, or bit-rot to distinguish strategic trade-offs from entropy.
- Measure cycle time trends: slowing velocity in a stable team size signals growing debt.

## Common Misreadings
Engineers sometimes use the term to describe any code they find ugly, which dilutes the metaphor into a generic quality complaint. Technical debt is specifically about the economic trade-off between speed now and cost later, not about aesthetic preference. Another misreading assumes all debt is bad; in reality, strategic debt taken to hit a market window can be a rational business decision, provided the repayment plan exists. The most damaging misunderstanding is treating debt as an inevitability that never requires action, which turns the metaphor from a management tool into an excuse.

## Interactions
Technical Debt is the economic framing that connects the Boy Scout Rule (micro-repayment) to the Broken Windows Theory (visible neglect signals lower standards). Lehman's Laws predict that evolving systems require increasing effort; unpaid debt accelerates that curve beyond Lehman's baseline. Sturgeon's Law applies to debt repayment priorities: most debt items are low-impact, so focus repayment on the vital few that cause the highest friction. YAGNI reduces debt by preventing speculative features that become unmaintained baggage.

---

*Based on: Cunningham, OOPSLA (1992)*
