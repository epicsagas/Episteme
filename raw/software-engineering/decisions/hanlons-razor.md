# Hanlon's Razor

## Statement
Hanlon's Razor advises that when someone's actions cause harm, one should favor explanations based on ignorance, fatigue, or process failure over explanations that assume deliberate hostility. The heuristic does not deny that malice exists; it simply assigns it lower prior probability than mundane causes in most everyday situations.

## Origin
The aphorism is attributed to Robert J. Hanlon, who submitted it for inclusion in a joke book compiled by Arthur Bloch in 1980. Variants of the idea appeared decades earlier in science fiction, notably in Robert A. Heinlein's 1941 short story "Logic of Empire," which expressed a similar sentiment about incompetence outlasting malice. The principle entered public-domain idiom and has since become a staple of incident-response culture in high-reliability organizations.

## Software Implications
When a deployment breaks production at 2 a.m., the instinct to blame a careless colleague is both human and counterproductive. Hanlon's Razor redirects attention toward systemic contributors: missing automated tests, ambiguous runbooks, insufficient monitoring, or a CI pipeline that allowed the bad commit to merge. Treating the event as a process failure rather than a personal one leads to lasting fixes that protect the entire team.

The principle also shapes team dynamics. Code review cultures that assume good intent produce more constructive feedback and higher psychological safety than cultures that default to suspicion. When a reviewer writes "this function is redundant," the author who assumes the comment is helpful rather than hostile responds with curiosity instead of defensiveness. Blameless postmortem practice at companies such as Etsy and Google is essentially institutionalized Hanlon's Razor: every incident investigation begins by ruling out negligence before investigating process gaps.

Security engineering is a notable boundary case. While Hanlon's Razor applies to internal failures, external threats are often genuinely malicious, and the heuristic must be set aside when analyzing adversarial behavior such as credential stuffing or insider data exfiltration.

## Practical Guidance
- Begin every incident review by asking "what process allowed this to happen?" before asking "who allowed this to happen?"
- Encode this assumption into team norms: review comments describe observations, not judgments of intent.
- Distinguish internal failures, where Hanlon's Razor is a strong default, from external threat analysis, where adversarial thinking is required.

## Common Misreadings
Some teams interpret Hanlon's Razor as a prohibition against holding individuals accountable. The razor cautions against assuming malicious intent, not against addressing repeated negligence. A developer who consistently skips code review is not malicious, but the pattern still requires corrective action. Another misreading applies the razor universally, including domains where adversarial behavior is the norm, such as information security, where assuming incompetence can leave systems exposed.

## Interactions
Hanlon's Razor complements Occam's Razor by extending the preference for simple explanations into the social domain: incompetence is a simpler hypothesis than conspiracy. It supports the Sunk Cost Fallacy's corrective framing, because teams that blame individuals rather than processes are more likely to double down on punitive measures instead of improving systems. The Map Is Not the Territory is relevant because a team's mental model of a colleague's intentions is itself a map, and Hanlon's Razor reminds us to prefer the map with fewer unfounded assumptions.

---

*Based on: Public domain*
