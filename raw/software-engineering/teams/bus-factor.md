# Bus Factor

## Statement
Bus Factor is the minimum number of team members whose simultaneous departure would halt a project or render a codebase unmaintainable. A bus factor of one means that a single person holds irreplaceable knowledge; a higher number indicates broader knowledge distribution and greater organizational resilience.

## Origin
The concept emerged from software engineering community discussions in the late 1990s and early 2000s as a vivid metaphor for key-person risk. The term gained currency on mailing lists and conference talks as engineers shared stories of projects that collapsed when a critical contributor left. Unlike the other laws in this collection, no single author or academic paper introduced the idea; it crystallized organically from practitioner experience and became standard vocabulary in engineering risk management.

## Software Implications
Every engineering organization has implicit bus factors across its subsystems. The database expert who is the only person who understands the replication topology. The frontend engineer who wrote the entire build pipeline and never documented it. The principal architect whose mental model of the authentication flow is the only authoritative source. When any of these people resigns, takes extended leave, or is reassigned, the team must reconstruct knowledge from scratch, often by reading code that was never designed to be self-documenting.

Startups routinely operate with a bus factor of one for their entire system. The founding engineer built everything, and no one else has end-to-end context. This is rational during early stages when speed matters more than resilience, but it becomes a critical business risk as the company grows and the system becomes central to revenue. Investors and acquirers routinely assess bus factor during due diligence because it directly affects the sustainability of the technical asset.

The bus factor also applies to non-code knowledge: operational runbooks, deployment procedures, vendor relationships, and incident response patterns. A team may have five engineers who can modify the codebase but only one who knows how to perform a production database migration safely.

## Practical Guidance
- Conduct regular bus factor audits by asking, for each critical subsystem, "how many people can debug this in production at 3 AM?"
- Require that every production system has at least two documented owners and that no single person is the sole reviewer of their own changes.
- Rotate on-call responsibilities so that knowledge of operational procedures spreads across the team.
- Write runbooks for every critical operational procedure, and test them by having someone other than the author follow them.

## Common Misreadings
A high bus factor does not guarantee project health. A team of ten where everyone has shallow familiarity with every subsystem has a nominally high bus factor but may still struggle when problems require deep expertise. The goal is deep knowledge held by multiple people, not superficial knowledge held by many.

Some teams misinterpret bus factor as an argument against specialization. Specialization is valuable; the problem is not that one person knows a subsystem deeply, but that only one person does. The solution is to pair specialists with apprentices who build the same depth over time.

## Interactions
Bus Factor intersects with Price's Law because the productive core identified by Price's Law often coincides with the critical few whose departure would cripple the project. It is aggravated by the Peter Principle when key experts are promoted away from the technical work where their knowledge is most needed. Putt's Law compounds the risk when managers who lack technical depth are the ones responsible for succession planning. Conway's Law provides a structural lens: if the org chart concentrates knowledge in one team, the bus factor for that team's domain will be low regardless of individual efforts to share knowledge.

---

*Based on: Fitzpatrick, "The Truck Number" (ApacheCON/Google Tech Talk, 2010)*
