# Confirmation Bias

## Statement
Confirmation Bias is the systematic tendency to seek, interpret, and remember information that supports pre-existing beliefs while ignoring, discounting, or forgetting evidence that contradicts them. It operates at the levels of attention, interpretation, and memory, making it one of the most pervasive and resilient cognitive biases.

## Origin
The bias was identified through experimental work by Peter Wason in the 1960s, most famously in his "2-4-6" number-sequence task, which demonstrated that people test hypotheses by searching for confirming examples rather than attempting to falsify them. Leon Festinger's theory of cognitive dissonance (1957) provided a related framework: when new evidence conflicts with an existing belief, the discomfort motivates the believer to reject the evidence rather than revise the belief. Decades of replication have established confirmation bias as a robust finding across cultures, expertise levels, and domains.

## Software Implications
When an engineer believes that a particular framework is the right choice for a project, confirmation bias shapes every subsequent evaluation. Benchmark results that favor the framework are highlighted, while results that favor alternatives are scrutinized for methodological flaws. Documentation that confirms the engineer's preference is read carefully; documentation that challenges it is skimmed or dismissed as outdated.

In debugging, confirmation bias causes developers to accumulate evidence for their initial hypothesis rather than actively seeking disconfirmation. If an engineer suspects a network issue is causing intermittent failures, they may spend hours examining packet captures while ignoring log lines that point to a race condition in application code. The discipline of forming a falsifiable hypothesis and then attempting to break it is a direct countermeasure.

Architecture reviews and technology evaluations are particularly vulnerable because they involve multiple stakeholders with strong prior beliefs. Without structured evaluation criteria established before the analysis begins, each stakeholder walks away from the same evidence set with their original opinion reinforced.

## Practical Guidance
- Define evaluation criteria and scoring rubrics before examining evidence, so that the criteria are not unconsciously shaped by preliminary findings.
- Assign a designated devil's advocate or red-team role during design reviews to actively argue against the prevailing hypothesis.
- In debugging, write down a falsifiable prediction before each diagnostic step; if the prediction fails, abandon the hypothesis rather than refining it.

## Common Misreadings
Confirmation bias is sometimes confused with simply being wrong. A person can hold an incorrect belief without exhibiting confirmation bias if they arrived at it through genuine misinterpretation rather than selective evidence processing. Conversely, a person can hold a correct belief and still exhibit confirmation bias by attending only to supporting evidence and ignoring valid but contradictory data. The bias describes the process of evidence handling, not the accuracy of the conclusion.

## Interactions
Confirmation Bias works synergistically with the Dunning-Kruger Effect: overconfident individuals are both more likely to hold strong prior beliefs and less likely to recognize disconfirming evidence. It fuels the Sunk Cost Fallacy by causing decision-makers to overweight confirming signals and underweight disconfirming ones as a project's costs mount. The Map Is Not the Territory is closely related because a mental model is itself a kind of map, and confirmation bias ensures that contradictory territory is filtered out. Inversion offers a practical antidote: deliberately asking "what evidence would prove me wrong?" forces engagement with disconfirming data.

---

*Based on: Cognitive psychology*
