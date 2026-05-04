# Sturgeon's Law

## Statement
Sturgeon's Law asserts that ninety percent of everything is mediocre, and by implication, only ten percent represents genuinely excellent work. In software engineering, the law warns that most available code, libraries, tools, and design patterns are undistinguished, and discerning selection is essential.

## Origin
Science fiction author Theodore Sturgeon formulated the observation around 1953 in response to critics who dismissed the entire genre by citing its worst examples. Sturgeon countered that ninety percent of science fiction was indeed terrible, but ninety percent of everything is terrible; the remaining ten percent in any field is what matters. The principle generalizes naturally to any creative or intellectual domain, including software, where the proliferation of open-source packages, framework choices, and internal codebases creates a vast selection problem.

## Software Implications
When a developer evaluates npm packages for a specific function, the majority of candidates will have poor documentation, sparse tests, inconsistent APIs, and dormant maintainers. The ten percent that are well-designed, thoroughly tested, actively maintained, and clearly documented are the ones worth adopting. Selecting from the ninety percent because it is convenient or appears first in search results saddles the project with hidden maintenance cost.

The law applies with equal force inside organizations. Not every internal service, shared library, or architectural decision meets a high standard. Teams that accept every internal dependency without evaluation accumulate the same quality debt as teams that choose poor external packages. The discipline of evaluating dependencies against quality criteria — test coverage, API stability, release cadence, and documentation quality — applies regardless of origin.

Sturgeon's Law also governs code output. A developer produces mediocre code under time pressure, distraction, or insufficient domain understanding. The ten percent of sessions where focus, domain clarity, and deliberate practice align produce disproportionately valuable code. Recognizing this distribution helps teams invest in conditions that promote high-quality output: uninterrupted focus blocks, clear requirements, and thorough design review before implementation.

## Practical Guidance
- Evaluate every dependency against a written quality rubric before adoption; the rubric should include test coverage, documentation, release frequency, and maintainer responsiveness.
- Study widely praised codebases to develop taste for the ten percent; reading excellent code is a skill that improves with practice.
- Reject the temptation to settle for the first adequate solution when a better one is discoverable with modest additional effort.
- Invest in code review culture that distinguishes acceptable from excellent, not just correct from incorrect.

## Common Misreadings
Some interpret the law as elitist gatekeeping that dismisses most contributions, which misreads Sturgeon's intent. The observation is statistical, not judgmental: most work is average by definition, and recognizing that fact enables better selection, not contempt for the average. Another misreading uses the law to justify never improving one's own work, arguing that ninety percent will always be mediocre. The productive reading is that excellence is achievable but requires disproportionate effort and deliberate cultivation. A third error applies the ninety/ten split as a precise threshold rather than a rough heuristic for "most" versus "the exceptional few."

## Interactions
Sturgeon's Law explains why the Boy Scout Rule and Broken Windows Theory matter: without active effort, code tends toward the ninety percent. It connects to YAGNI because speculative features tend to fall into the mediocre majority, while features driven by actual need have a higher probability of being well-designed. The law reinforces the value of Linus's Law: more reviewers increase the chance that the excellent ten percent of suggestions rise to the top. Technical Debt is partly the accumulation of decisions made from the ninety percent without investing in the evaluation needed to find the ten.

---

*Based on: Sturgeon, science fiction criticism (1953)*
