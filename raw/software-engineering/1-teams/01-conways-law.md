# Conway's Law

## Statement
Conway's Law asserts that any system an organization produces will replicate the communication pathways of that organization. Teams separated by reporting lines, geography, or process boundaries inevitably build software modules separated by those same boundaries.

## Origin
Melvin Conway, a programmer and consultant, advanced this observation in a 1968 article titled "How Do Committees Invent?" submitted to Datamation magazine. Conway argued that the design architecture of any system is constrained by the social architecture of the group designing it. The insight gained widespread recognition after Fred Brooks cited it in The Mythical Man-Month, and decades of empirical research have since confirmed the correlation between organizational structure and software modularity.

## Software Implications
When a company divides its engineering department into a frontend team, a backend team, and a database team, the resulting system will expose those same seams through rigid API layers and integration points. Each team optimizes locally for its own domain, producing interfaces that reflect organizational convenience rather than user needs. Conversely, when a company organizes around cross-functional product squads, the software tends toward cohesive, vertically sliced services that align with business capabilities.

The so-called Inverse Conway Maneuver exploits this relationship in reverse: leadership deliberately restructures teams to elicit a desired architectural outcome. A migration from a monolith to microservices, for instance, usually fails if the team structure remains monolithic. Amazon's transition to service-oriented architecture succeeded precisely because the company reorganized into small, autonomous "two-pizza" teams, each responsible for a single service boundary. The architecture followed the org chart, not the other way around.

API design is another observable consequence. Public interfaces between modules tend to be clean and well-documented where communication between teams is frequent and informal, but they become bloated and inconsistent where teams communicate through formal channels alone.

## Practical Guidance
- Before rearchitecting a system, restructure the teams first; software boundaries follow organizational boundaries.
- Limit the number of people who must coordinate across any single interface to keep that interface coherent.
- Conduct architecture reviews that explicitly examine whether team topology matches the intended module topology.
- Use feature teams rather than component teams when you want vertically integrated software slices.

## Common Misreadings
Some interpret Conway's Law as fatalism, believing organizational structure rigidly determines architecture with no room for deliberate design. In practice, the law describes a strong tendency, not an ironclad constraint. Skilled architects can counteract organizational gravity through explicit interface contracts and cross-team design sessions. The danger lies in ignoring the law entirely and expecting architectural purity from a misaligned team structure.

Another common error is assuming the law applies only to large organizations. Even a five-person startup exhibits Conway effects: the two founders who sit together will build tightly coupled modules, while the remote contractor will produce a cleanly separated component with a narrow integration surface.

## Interactions
Conway's Law amplifies the communication overhead described by Brooks's Law, because team boundaries determine where that overhead concentrates. It interacts with Dunbar's Number as organizations grow past 150 people, forcing formal communication structures that produce more rigid system boundaries. The Ringelmann Effect further compounds the problem: large teams produce less per-person output, and the resulting architecture reflects that diffused effort. Price's Law predicts which individuals within those team boundaries will drive the majority of architectural decisions.

---

*Based on: Conway, "How Do Committees Invent?" (Datamation, 1968)*
