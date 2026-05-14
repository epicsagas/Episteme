# Price's Law

## Statement
Price's Law states that in any collaborative domain, the square root of the total number of participants produces half of all output. In a 100-person engineering department, roughly 10 individuals generate 50 percent of the results, a pattern that becomes more extreme as organizations grow.

## Origin
Derek J. de Solla Price, a physicist and pioneer of scientometrics, identified this relationship through his study of scientific publication patterns in the 1960s. Price observed that the distribution of academic papers followed a heavily skewed curve: a small fraction of researchers authored a disproportionate share of the literature. He formalized the square-root relationship in his book Little Science, Big Science (1963), extending an insight that shares mathematical properties with the Pareto principle but provides a more precise scaling function tied to group size.

## Software Implications
Engineering organizations exhibit this distribution consistently. In a 25-person team, about 5 people will drive half the architectural decisions, code contributions, and production incident resolutions. The remaining 20 contribute, but their aggregate output equals that of the core 5. This is not a sign of dysfunction; it reflects natural variation in skill, experience, domain knowledge, and intrinsic motivation.

Open-source projects demonstrate Price's Law in stark terms. A project with 400 contributors will typically have 20 maintainers who handle the majority of code reviews, design decisions, and issue triage. The long tail of contributors submits occasional bug fixes or documentation improvements, which is valuable, but the project's trajectory is set by the square-root core.

The law has uncomfortable implications for management. It means that losing one top contributor from a 100-person organization removes far more productive capacity than losing one average contributor. It also means that hiring efforts aimed at doubling team size must contend with the mathematical reality that the productive core will only grow by about 40 percent. Retaining the small number of high-output individuals becomes disproportionately important.

## Practical Guidance
- Identify the square-root core of your organization and ensure they are supported, retained, and not overloaded with managerial burden.
- Design mentoring programs that expose rising contributors to the problems the core works on, expanding the effective core over time.
- Avoid structuring compensation and recognition as though all contributors produce equal output.
- Use this distribution to inform hiring strategy: one exceptional hire contributes more than several average ones.

## Common Misreadings
Price's Law is sometimes misused to justify neglecting the majority of team members. The square root of participants produces half the output, but the other half still matters; products ship because of the aggregate effort. Treating non-core contributors as interchangeable or expendable demoralizes the team and shrinks the talent pipeline from which future core contributors emerge.

Another error is treating the law as a precise predictive tool. The square-root relationship is a statistical tendency observed across populations, not a guarantee that exactly ten people in a hundred-person company will produce exactly 50 percent of the output. Real distributions fluctuate, and organizational context matters.

## Interactions
Price's Law intersects with the Peter Principle because high-output individual contributors are often promoted into management roles where they may be less effective, reducing the productive core. It reinforces the Bus Factor risk: if the square-root core shares critical knowledge that the majority does not hold, the project is fragile. Putt's Law describes the organizational dynamic that pulls productive experts away from hands-on work and into administrative roles. The Dilbert Principle further erodes the productive core if incompetent employees are promoted to management and then direct the remaining contributors inefficiently.

---

*Based on: Price, "Little Science, Big Science" (Columbia University Press, 1963)*
