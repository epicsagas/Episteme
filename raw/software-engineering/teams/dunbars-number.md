# Dunbar's Number

## Statement
Dunbar's Number identifies a cognitive ceiling of roughly 150 stable social relationships that any one person can maintain simultaneously. Beyond this threshold, interpersonal bonds weaken and organizations must substitute formal policies, hierarchies, and documentation for the trust-based coordination that small groups rely on.

## Origin
British anthropologist Robin Dunbar established this limit through research on primate neocortex size and its correlation with social group size, published in 1992. By extrapolating the relationship between brain structure and social group size across primate species to humans, Dunbar arrived at the approximate figure of 150. Subsequent field studies of human communities, from hunter-gatherer bands to military units and corporate divisions, have repeatedly confirmed that groups exceeding this size undergo qualitative changes in social cohesion.

## Software Implications
Engineering organizations feel this limit acutely. A startup with 40 engineers can coordinate through hallway conversations, a shared Slack channel, and mutual awareness of who knows what. When the same company reaches 200 engineers, those informal mechanisms break down. People no longer know whom to ask, duplicated work emerges across teams, and decisions made in one corner of the organization contradict decisions made in another. The organization must then invest in formal structures: architectural review boards, written design documents, decision logs, and explicit ownership assignments.

Team sizing is another direct application. Amazon's "two-pizza team" heuristic and the Agile guideline of five to nine members per team both operate well below Dunbar's Number. Within a small team, each member can maintain accurate mental models of what every other member is doing, which enables the high-bandwidth coordination that complex software development demands. When a single team grows past roughly 15 members, coordination costs begin to dominate productive output, and the group naturally fragments into sub-teams that communicate through more formal channels.

The number also explains why open-source projects with thousands of contributors remain governed by a core group of 10 to 30 maintainers. The maintainer community stays within Dunbar's limit, preserving the trust-based decision-making that keeps the project coherent, while the long tail of casual contributors interacts with the project through standardized processes rather than personal relationships.

## Practical Guidance
- When your engineering organization approaches 150 people, proactively introduce documentation standards, decision logs, and architectural review processes before informal coordination collapses.
- Keep individual teams between 5 and 9 members; split them when they exceed 12.
- Structure cross-team dependencies to minimize the number of people any one engineer must know personally to do their job effectively.
- Invest in onboarding programs that accelerate relationship-building, because new hires must integrate into the social graph before they can be fully productive.

## Common Misreadings
Dunbar's Number is not a single fixed value of exactly 150. Dunbar himself described a series of concentric layers: roughly 5 intimate contacts, 15 close friends, 50 casual friends, and 150 meaningful contacts, with less stable layers extending to 500 and 1500. Applying the strict 150 figure to every context misses the nuance that different group functions operate at different layers.

Another error is treating Dunbar's Number as an argument against large organizations. The limit constrains the size of a cohesive social group, not the size of a company. Large organizations function precisely because they decompose into smaller units connected by formal processes rather than personal relationships.

## Interactions
Dunbar's Number sets the stage for Conway's Law: as organizations grow past the cognitive limit, formal team boundaries harden, and the software architecture mirrors those boundaries. It magnifies Brooks's Law because communication overhead scales with the number of interpersonal channels that must be maintained. The Ringelmann Effect becomes more pronounced as group size crosses Dunbar thresholds, because social loafing increases when individuals can no longer track every group member's contribution. Price's Law operates within the subgroups below the Dunbar threshold, concentrating productive output among a few key individuals in each layer.

---

*Based on: Dunbar, "Neocortex size as a constraint on group size in primates" (Journal of Human Evolution, 1992)*
