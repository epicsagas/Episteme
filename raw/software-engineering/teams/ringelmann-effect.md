# The Ringelmann Effect

## Statement
The Ringelmann Effect describes the observed decline in individual effort as group size increases. Each additional member of a team tends to contribute less than the previous one, producing a gap between a group's theoretical capacity and its actual output.

## Origin
French agricultural engineer Maximilien Ringelmann documented this phenomenon in 1913 through experiments in which participants pulled on a rope, both individually and in groups. He found that groups of two achieved only 93 percent of their expected combined force, groups of three achieved 85 percent, and groups of eight achieved just 49 percent. Later social psychologists, notably Bibb Latane and colleagues in the 1970s and 1980s, replicated the finding and identified two causal mechanisms: coordination loss, where members fail to synchronize their efforts efficiently, and motivation loss, where individuals exert less effort because their personal contribution is less visible.

## Software Implications
In software teams, the Ringelmann Effect manifests as social loafing during collaborative tasks. A pair of developers working on a feature typically sustain high individual engagement because each person's contribution is immediately visible to the other. Expand the same task to a team of eight and several dynamics emerge: some members coast, assuming others will carry the load; the group spends increasing time in coordination meetings rather than producing code; and ownership of specific subtasks becomes diffuse.

Code review provides a clear example. When every team member is assigned as a reviewer on every pull request, the responsibility to review thoroughly diffuses across the group and reviews become superficial or delayed. Each person assumes someone else will catch the bug. Teams that assign a primary and secondary reviewer to each change, keeping the reviewing group small, consistently produce more thorough reviews with faster turnaround.

Standups and planning meetings illustrate the coordination loss component. A five-person daily standup finishes in ten minutes with each person contributing actionable updates. A fifteen-person standup balloons to thirty minutes, much of it consumed by context that is irrelevant to most attendees, and several participants mentally disengage.

## Practical Guidance
- Keep working groups small and give each member clearly distinguishable responsibilities.
- Assign primary ownership of tasks rather than sharing ownership across large groups.
- Limit code review assignments to two or three reviewers per pull request.
- Split large teams into smaller, focused sub-teams with distinct mandates before coordination costs erode individual output.

## Common Misreadings
The Ringelmann Effect does not imply that teams are inherently less productive than individuals. Total group output generally still increases with size, just at a diminishing rate per person. The effect warns against assuming that doubling a team doubles output, not against teamwork itself.

Some managers misapply the effect by reducing all team sizes to two, ignoring the fact that certain tasks genuinely require diverse expertise or parallel workstreams. The goal is to size teams appropriately for the task at hand, not to minimize team size in all cases.

## Interactions
The Ringelmann Effect compounds Brooks's Law: adding people to a late project not only increases communication overhead but also dilutes per-person effort. It reinforces Conway's Law by incentivizing the creation of smaller, more focused teams whose module boundaries reflect the team structure. Price's Law operates as a counterpoint, predicting that even within a loafing group, a small minority of members will still produce the majority of the output. Dunbar's Number provides the broader context: the Ringelmann Effect intensifies as group size crosses the cognitive thresholds where interpersonal accountability breaks down.

---

*Based on: Ringelmann, "Recherches sur les moteurs animés: Travail de l'homme" (Ann. Inst. Nat. Agronomique, 1913)*
