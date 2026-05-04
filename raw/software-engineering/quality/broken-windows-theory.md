# Broken Windows Theory

## Statement
The Broken Windows Theory holds that visible signs of disorder in a codebase signal that quality standards have lapsed, accelerating further decay. Tolerating small infractions normalizes carelessness and invites larger ones.

## Origin
Criminologists James Q. Wilson and George L. Kelling published the original Broken Windows Theory in The Atlantic in 1982, arguing that unmaintained urban environments encourage further vandalism and crime. Andrew Hunt and David Thomas adapted the concept for software in "The Pragmatic Programmer" (1999), arguing that one unchecked poor design decision in a codebase operates like a smashed window: it tells every subsequent developer that standards are optional. The analogy reframes code quality as a social-norm problem rather than purely a technical one.

## Software Implications
When a codebase contains one function with no documentation, unclear naming, and no tests, the next developer who adds a nearby function faces lower psychological resistance to skipping documentation and tests as well. The downward spiral is self-reinforcing: as quality degrades, the code becomes harder to understand, which makes careful work more difficult, which leads to sloppier changes. A team that permits TODO comments to accumulate without resolution eventually treats them as invisible. Build warnings that go unaddressed train developers to ignore the build output entirely.

The countermeasure is immediate repair. When a code review catches a style violation, the fix ships before merge. When a linter flags a new category of warning, the team addresses the existing instances before moving on. High-functioning teams set a "zero tolerance" baseline: the build stays green, warnings stay at zero, and every check-in leaves its immediate vicinity at least as clean as before. Google's mandatory code review process and the Linux kernel's strict submission guidelines are institutional embodiments of this principle.

The theory applies to process as well as code. A team that lets retrospective action items expire without action trains itself to treat retrospectives as theater. The broken window is not the missed action item but the visible proof that accountability is optional.

## Practical Guidance
- Treat every linter warning as a build error or fix existing warnings immediately upon enabling a new rule.
- Block merges on unresolved code review comments rather than deferring them to follow-up tickets.
- Schedule brief weekly cleanup sessions to eliminate accumulated TODO markers and dead code.
- Model quality standards visibly: when leads fix small issues, the team follows.

## Common Misreadings
Some interpret the theory as requiring perfection on every commit, which paralyzes throughput. The intent is not zero defects but zero tolerance for known defects left unaddressed in active areas. Another misreading applies the theory only to code aesthetics while ignoring architectural decay; a beautifully formatted module with a flawed abstraction is still a broken window. The principle targets visible signals of carelessness, not subjective judgments about elegance.

## Interactions
The Broken Windows Theory operates as the social-psychological complement to the Boy Scout Rule: the former explains why decay accelerates, the latter provides the practice that prevents it. It connects directly to Technical Debt because visible neglect is a form of debt that compounds through demoralization. Lehman's Laws predict that system complexity grows inevitably; Broken Windows explains why unmanaged complexity accelerates faster than Lehman's baseline. Linus's Law offers a remedy: more reviewers increase the chance that broken windows are spotted and fixed early.

---

*Based on: Wilson & Kelling, The Atlantic (1982)*
