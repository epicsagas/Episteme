# Dunning-Kruger Effect

## Statement
The Dunning-Kruger Effect describes a metacognitive failure in which individuals with limited skill in a domain overestimate their competence, while genuine experts tend to underestimate theirs. People who lack knowledge also lack the mental tools needed to recognize that gap, producing a confidence curve that peaks prematurely on the left side of the competence axis.

## Origin
Social psychologists David Dunning and Justin Kruger documented this bias in their 1999 paper "Unskilled and Unaware of It," published in the Journal of Personality and Social Psychology. They were inspired by the story of McArthur Wheeler, a bank robber who believed lemon juice rendered his face invisible to surveillance cameras. The paper demonstrated through a series of self-assessment experiments that bottom-quartile performers consistently overestimated their rank, while top-quartile performers modestly underestimated theirs.

## Software Implications
In engineering organizations the effect surfaces most visibly during estimation and architecture exercises. A developer who has completed a tutorial on distributed systems may feel qualified to design a globally consistent data layer, underestimating the fault-tolerance, latency, and operational challenges that a seasoned distributed-systems engineer would flag immediately. This misplaced confidence propagates into sprint commitments, technology-selection decisions, and hiring evaluations.

Conversely, senior engineers who have internalized how much they do not know may hedge their estimates or defer decisions longer than necessary. Teams that fail to account for both sides of the curve end up with overambitious roadmaps driven by the most confident voices rather than the most informed ones.

Code review is a natural counterweight. When an overconfident author submits a pull request, reviewers with deeper experience can expose hidden complexity before it ships. Retrospectives and blameless postmortems serve a similar corrective function by confronting teams with evidence that reality diverged from their earlier assumptions.

## Practical Guidance
- Pair overconfident contributors with domain mentors during design phases so hidden complexity surfaces early.
- Treat unusually high-confidence estimates as a risk signal and require supporting evidence before committing.
- Calibrate estimation accuracy by tracking predicted versus actual outcomes and sharing the data openly.

## Common Misreadings
A frequent misunderstanding treats the Dunning-Kruger Effect as a claim that inexperienced people are always wrong. The bias is about miscalibration of self-assessment, not about the quality of the output itself. A junior engineer may produce excellent code while still being unable to accurately judge how good that code is relative to an expert standard. Another misreading graphs confidence as a single smooth "Mount Stupid" curve from zero to mastery; the original research showed overlapping distributions, not a deterministic trajectory that every individual follows.

## Interactions
The Dunning-Kruger Effect amplifies the harm of Confirmation Bias, because overconfident individuals seek evidence that validates their inflated self-view and dismiss corrective feedback. It also interacts with the Hype Cycle: early in a technology's hype curve, practitioners with shallow exposure may project unrealistic confidence onto adoption plans. Occam's Razor offers a partial antidote, since simpler explanations tend to expose the limits of an argument faster than speculative complexity. The Sunk Cost Fallacy can compound the effect once an overconfident decision has been made, as admitting a mistake would require acknowledging the original overestimation.

---

*Based on: Kruger & Dunning, "Unskilled and Unaware of It" (J. Personality & Social Psychology, 1999)*
