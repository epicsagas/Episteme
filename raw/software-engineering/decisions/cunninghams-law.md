# Cunningham's Law

## Statement
Cunningham's Law observes that the most reliable way to elicit a correct answer from a group is to state a wrong answer with confidence, because people are far more motivated to correct a visible error than to respond to an open question. The mechanism exploits a human impulse that is stronger than the desire to help: the desire to be right in public.

## Origin
The principle is attributed to Ward Cunningham, the creator of the first wiki and a pioneer of extreme programming and agile methodologies. In the collaborative wiki culture of the 1990s and 2000s, participants discovered that pages with deliberate or accidental inaccuracies were rapidly corrected by other contributors, while pages that asked open questions often languished without response. The pattern was informal and anecdotal rather than the product of a controlled study, but it proved robust across many online communities.

## Software Implications
On engineering teams, Cunningham's Law suggests that posting a concrete but imperfect proposal generates more useful feedback than asking "what should we do?" A design document that takes a specific position invites reviewers to identify weaknesses, propose alternatives, and share relevant experience. A blank-slate request for input often produces silence because responders must do the creative work of generating a position before they can critique it.

In code review, the same dynamic applies. A pull request that includes a comment such as "I handled the retry logic this way, but I am not sure it covers all edge cases" is more likely to attract thorough review than one with no annotations. Reviewers are drawn to specific claims they can verify or refute.

The principle extends to debugging and learning. An engineer who shares a hypothesis about a bug's root cause -- even a tentative one -- often receives faster and more targeted corrections than one who posts a general plea for help. The wrong hypothesis gives experts a concrete statement to react to, and their correction is more precise than it would be in response to a vague question.

## Practical Guidance
- When seeking architectural feedback, write a concrete proposal with explicit trade-offs rather than asking for open-ended input; the proposal gives reviewers a target to react to.
- In on-call situations, share your working hypothesis in the incident channel even if you are unsure, because a specific claim accelerates collaborative diagnosis.
- In documentation, prefer assertive statements with citations over hedged language, because incorrect assertions get corrected faster than uncertain ones get clarified.

## Common Misreadings
Cunningham's Law is sometimes interpreted as advice to deliberately post misinformation, which is neither the intent nor a responsible practice. The principle describes an observed tendency in collaborative environments, not a license to deceive. Another misreading treats the law as a claim that asking questions is ineffective; questions work well when they are specific and when the audience is motivated. The law describes a relative difference in response rates, not an absolute prohibition on asking.

## Interactions
Cunningham's Law complements Inversion by providing a social mechanism for surfacing hidden knowledge: stating a wrong answer is a form of inverted questioning that reveals what not to do. It interacts with Confirmation Bias because an incorrect but confident statement can, in the wrong context, reinforce existing misconceptions if no expert is present to correct it. The Dunning-Kruger Effect is relevant because overconfident individuals may accidentally invoke Cunningham's Law by stating wrong answers with genuine conviction, and the quality of the corrections they receive depends on the expertise of their audience. Occam's Razor can help evaluate whether a correction is genuine: the simplest explanation for a correction is usually that the corrector has direct knowledge, not that they are guessing.

---

*Based on: Ward Cunningham, wiki culture (c. 2004)*
