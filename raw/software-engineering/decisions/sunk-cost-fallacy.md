# Sunk Cost Fallacy

## Statement
The Sunk Cost Fallacy is the tendency to continue investing effort, money, or time into a failing endeavor because of resources already spent, rather than evaluating the decision based solely on future costs and benefits. Past expenditure that cannot be recovered should have no bearing on forward-looking choices.

## Origin
The concept originates in behavioral economics and was formalized through the work of researchers such as Richard Thaler and Daniel Kahneman in the 1970s and 1980s. Their studies on prospect theory and loss aversion showed that people feel losses approximately twice as strongly as equivalent gains, creating a powerful psychological incentive to avoid admitting a loss by doubling down on the original commitment. The fallacy has since been replicated across cultures and domains, from business investment to personal relationships.

## Software Implications
Technology migrations are a prime vector for the Sunk Cost Fallacy. A team that has spent six months building a custom ORM on top of a legacy database may resist switching to a mature open-source alternative, arguing that abandoning the custom work would "waste" the effort already invested. In reality, the six months are gone regardless; the only relevant question is which option produces better outcomes from this point forward.

Rewrite decisions suffer the same distortion. A codebase that has accumulated years of technical debt often survives past the point of diminishing returns because stakeholders frame a rewrite as "throwing away" the existing investment. Rational analysis ignores sunk cost and compares the projected maintenance burden of the old system against the projected cost and risk of the replacement.

At a smaller scale, individual developers fall into the trap during debugging sessions. After hours spent pursuing a particular hypothesis, an engineer may resist abandoning it even when evidence points elsewhere, because switching feels like discarding the effort already spent.

## Practical Guidance
- When evaluating whether to continue or pivot, explicitly list only future costs and future benefits; strike any reference to past investment from the decision frame.
- Establish predefined kill criteria for projects before they begin, so that the decision to stop is anchored in measurable conditions rather than emotional attachment.
- Normalize the language of "pivoting" within team culture so that changing direction is seen as a rational adaptation rather than a confession of failure.

## Common Misreadings
A dangerous misreading treats every pivot as inherently rational and every continuation as a sunk-cost error. Some endeavors are genuinely close to completion, and the cost to finish is small relative to the value of finishing. Discerning the fallacy requires honest forward-looking analysis, not reflexive abandonment. Another misunderstanding equates sunk cost with total investment; the fallacy applies only to irrecoverable expenditure. If part of the investment can be repurposed -- such as reusable components from a cancelled project -- that salvageable fraction is not sunk and should be counted as a future asset.

## Interactions
The Sunk Cost Fallacy often co-occurs with Confirmation Bias: once committed to a course of action, people selectively notice evidence that supports continuing while discounting signals to stop. The Dunning-Kruger Effect can initiate the cycle, because overconfident estimates lead to large upfront investments that later become psychological anchors. The Hype Cycle intensifies the fallacy when organizations invest heavily in a technology at peak hype and then resist abandoning it during the trough of disillusionment. Inversion provides a useful countermeasure: asking "if we had not already invested in this, would we start today?" strips sunk cost from the evaluation.

---

*Based on: Thaler, "Toward a Positive Theory of Consumer Choice" (Behavioral economics, 1980)*
