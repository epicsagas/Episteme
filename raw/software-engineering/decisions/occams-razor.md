# Occam's Razor

## Statement
Occam's Razor holds that when multiple hypotheses explain the same observations, the one requiring the fewest unsupported assumptions should be preferred. It is a heuristic for prioritizing investigations, not a guarantee that the simplest explanation is always true.

## Origin
The principle is named after William of Ockham, a Franciscan friar and philosopher writing in the early fourteenth century. His formulation -- "entities should not be multiplied beyond necessity" -- was a methodological rule for theological and logical disputation, not a claim about the structure of reality. Earlier thinkers, including Aristotle and Ptolemy, expressed similar preferences for explanatory economy, but Ockham's name became permanently attached to the idea through centuries of scholarly citation.

## Software Implications
In debugging, Occam's Razor is the instinct that leads an engineer to check for a null pointer before suspecting a cosmic-ray bit flip. When a web service returns 500 errors, the likeliest explanations are a misconfigured environment variable, an exhausted connection pool, or an unhandled edge case -- not an obscure runtime bug or a compromised dependency. Investigating the simplest candidates first converges on root cause faster and at lower cost.

In system design, the razor argues against premature generalization. A data pipeline that handles one file format does not need an abstraction layer for five hypothetical future formats until those formats actually materialize. Frameworks like YAGNI ("You Aren't Gonna Need It") and KISS ("Keep It Simple, Stupid") are Occam's Razor translated into engineering maxims: add complexity only when evidence demands it.

The razor also applies to retrospective analysis. When a project misses its deadline, the simplest explanation is typically that the scope was underestimated, not that external forces conspired against the team. Choosing the fewer-assumption hypothesis keeps the improvement conversation grounded in actionable corrections.

## Practical Guidance
- When debugging, enumerate candidate causes and investigate them in order of assumption count, starting with the fewest.
- During design reviews, challenge every abstraction layer by asking what concrete requirement it satisfies today.
- Apply the razor symmetrically: if the simplest hypothesis is disproven, advance to the next-simplest rather than leaping to the most complex.

## Common Misreadings
Occam's Razor is sometimes paraphrased as "the simplest explanation is correct," which misrepresents it as a truth criterion rather than a preference rule. In reality, the universe frequently turns out to be more complex than the simplest model suggests. Another error is equating "fewest assumptions" with "most familiar"; a comfortable explanation and a parsimonious one are not always the same thing, and developers sometimes reject unfamiliar but well-evidenced hypotheses in favor of familiar but speculative ones.

## Interactions
Occam's Razor is the intellectual sibling of Hanlon's Razor, which applies the same parsimony principle to human behavior by preferring incompetence over conspiracy. It tempers the Dunning-Kruger Effect because overconfident individuals tend to construct elaborate theories that a simpler analysis would dismiss. The Lindy Effect can serve as a complement: technologies that have survived a long time have already shed unnecessary complexity, making them simpler to reason about than newer alternatives whose assumptions remain untested. Confirmation Bias undermines Occam's Razor when engineers selectively weight evidence to make their preferred hypothesis appear simpler than it is.

---

*Based on: William of Ockham, Lex Parsimoniae (c. 1287-1347)*
