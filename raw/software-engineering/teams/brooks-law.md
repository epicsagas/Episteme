# Brooks's Law

## Statement
Brooks's Law declares that adding people to a software project that is already behind schedule will make it even later. The additional communication and training overhead consumed by onboarding new contributors outweighs any productive output they might contribute in the short term.

## Origin
Frederick Brooks formulated this principle in The Mythical Man-Month, published in 1975 based on his experience managing IBM's OS/360 operating system project. Brooks introduced the concept of the "mythical man-month" to expose the fallacy of treating person-time as a fungible resource that can be exchanged freely. The book became a foundational text in software engineering management and remains widely cited in project planning literature.

## Software Implications
The core mechanism is communication overhead. If a team of N developers maintains N(N-1)/2 communication channels, adding even one person increases those channels by N. A four-person team has 6 channels; a ten-person team has 45. Each new member must also absorb domain knowledge from existing members, who are thereby diverted from productive work precisely when the project can least afford it.

This law exposes the person-month as a dangerous unit of estimation. A task estimated at twelve person-months cannot necessarily be completed in one month by twelve people. Many software activities involve sequential dependencies, shared state, and creative problem-solving that resist parallelization. Debugging a race condition, for instance, does not proceed faster with more debuggers; the bottleneck is understanding, not labor.

In practice, the law is most visible during crisis responses. A project slipping by three months prompts management to double the team, expecting a halving of the remaining schedule. Instead, the original developers spend their next month writing onboarding documents, pairing with newcomers, and attending additional sync meetings. Velocity drops before it eventually recovers, often after the original deadline has already passed.

## Practical Guidance
- When a project is late, investigate and remove blockers before considering headcount increases.
- If you must add people, bring them onto tasks that are genuinely independent and well-bounded.
- Invest in documentation and onboarding infrastructure continuously, not just during crises.
- Track communication overhead explicitly as team size grows, and split teams before coordination costs dominate.

## Common Misreadings
Brooks's Law is sometimes treated as an absolute prohibition on adding staff, which is not what Brooks intended. He himself noted an exception for tasks that are fully decomposable and where new members are already trained. The law is a warning about the cost of late additions, not a blanket rule against growing teams. It also does not apply when the existing team is understaffed relative to genuinely parallelizable work.

Another misreading is using Brooks's Law to excuse poor management. A team that is late because of unclear requirements or technical debt will not recover simply by holding headcount constant and working harder. The law describes one specific failure mode, not all of them.

## Interactions
Brooks's Law is reinforced by the Ringelmann Effect, which explains the per-person productivity drop as groups enlarge. It connects to Conway's Law through communication structure: adding people changes the org chart, which changes the architecture. Parkinson's Law compounds the damage, because the expanded team will find ways to fill whatever schedule extension results. Dunbar's Number provides the cognitive ceiling that makes communication overhead feel overwhelming once team size exceeds what individuals can track socially.

---

*Based on: Brooks, "The Mythical Man-Month" (Addison-Wesley, 1975)*
