# Linus's Law

## Statement
Linus's Law states that given a sufficiently large and diverse group of reviewers, every software defect becomes trivially discoverable. The breadth of examination, not the depth of any single auditor, determines defect detection rates.

## Origin
Eric S. Raymond formulated the aphorism in "The Cathedral and the Bazaar" (1997), attributing the insight to the development culture Linus Torvalds created around the Linux kernel. Raymond observed that Linux's open development model, where anyone could read and comment on patches, produced remarkably reliable code despite a high rate of change. The law formalizes the empirical observation that distributed scrutiny outperforms centralized review for finding subtle defects.

## Software Implications
A race condition in concurrent code might elude the original author because their mental model assumes a particular execution order. A reviewer with experience in concurrent systems spots the hazard immediately because they have trained themselves to look for interleaving assumptions. A third reviewer with security expertise notices that the same code exposes a timing side-channel. No single reviewer is likely to catch all three issues, but a diverse pool makes each category of bug shallow.

The law explains why mandatory code review is among the most effective quality practices in industry data. It also explains why open-source projects with large contributor bases discover vulnerabilities faster than proprietary projects of similar complexity, provided the code is actually being read. Bug bounty programs operationalize Linus's Law by expanding the reviewer pool to include external security researchers who bring adversarial perspectives that internal teams lack.

The law has limits: reviewers must be competent, motivated, and actually reading the code rather than rubber-stamping. A review process that approves every change in under five minutes provides the illusion of scrutiny without its substance. The diversity of reviewers matters as much as the count; a hundred reviewers with identical experience will miss the same categories of bugs.

## Practical Guidance
- Require at least two reviewers for changes touching critical paths, with at least one reviewer from a different sub-team.
- Rotate review assignments so that no single file is always reviewed by the same person.
- Document reviewer expectations: reviewers should describe what they checked, not just approve.
- Invest in code readability through documentation and clear naming so that reviewers can be effective quickly.

## Common Misreadings
The most common misreading equates "many eyeballs" with "any eyeballs." Raw review count without reviewer competence, diversity, or effort produces no improvement. Another error is assuming the law means code does not need automated testing because reviewers will find bugs; review and testing are complementary, not substitutable. A subtle misunderstanding treats the law as a claim that all bugs are easy to fix once found, when the law actually addresses discoverability, not fix difficulty.

## Interactions
Linus's Law reinforces the Broken Windows Theory because more reviewers mean broken windows are identified sooner. It complements the Testing Pyramid: automated tests catch mechanical regressions while human reviewers catch design and semantic errors. The Pesticide Paradox applies to review as well as testing; rotating reviewers prevents stale patterns of attention. Murphy's Law motivates expanding review diversity, since more failure modes require more perspectives to anticipate them.

---

*Based on: Raymond, "The Cathedral and the Bazaar" (1997)*
