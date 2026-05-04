# 56 Laws of Software Engineering - English Original Version

This directory contains the English original versions of the 56 Laws of Software Engineering, organized by category.

## 📑 Table of Contents

### 1. Teams (9 laws)
Principles related to team organization, management, and personnel distribution.

- [01 Conway's Law](./1-teams/01-conways-law.md) - Organizations design systems that mirror their own communication structure
- [02 Brooks's Law](./1-teams/02-brooks-law.md) - Adding manpower to a late software project makes it later
- [03 Dunbar's Number](./1-teams/03-dunbars-number.md) - Cognitive limit of about 150 stable relationships one can maintain
- [04 The Ringelmann Effect](./1-teams/04-ringelmann-effect.md) - Individual productivity decreases as group size increases
- [05 Price's Law](./1-teams/05-prices-law.md) - The square root of total participants does 50% of the work
- [06 Putt's Law](./1-teams/06-putts-law.md) - Those who understand technology don't manage it; managers don't understand it
- [07 Peter Principle](./1-teams/07-peter-principle.md) - In hierarchies, employees tend to rise to their level of incompetence
- [08 Bus Factor](./1-teams/08-bus-factor.md) - Minimum team members whose loss would put a project in serious trouble
- [09 Dilbert Principle](./1-teams/09-dilbert-principle.md) - Companies promote incompetent employees to management to limit damage

### 2. Planning & Estimation (6 laws)
Principles related to project scheduling, estimation, and planning.

- [10 Premature Optimization](./2-planning/10-premature-optimization.md) - Premature optimization is the root of all evil
- [11 Parkinson's Law](./2-planning/11-parkinsons-law.md) - Work expands to fill the time available for completion
- [12 The Ninety-Ninety Rule](./2-planning/12-ninety-ninety-rule.md) - First 90% of code takes 90% of time; remaining 10% takes the other 90%
- [13 Hofstadter's Law](./2-planning/13-hofstadters-law.md) - It always takes longer than expected, even accounting for this law
- [14 Goodhart's Law](./2-planning/14-goodharts-law.md) - When a measure becomes a target, it ceases to be a good measure
- [15 Gilb's Law](./2-planning/15-gilbs-law.md) - Anything needing quantification can be measured better than not measuring it

### 3. Architecture (9 laws)
Principles related to software system design and structure.

- [16 Hyrum's Law](./3-architecture/16-hyrums-law.md) - With sufficient API users, all observable behaviors become depended upon
- [17 Gall's Law](./3-architecture/17-galls-law.md) - Complex systems that work evolved from simple systems that worked
- [18 The Law of Leaky Abstractions](./3-architecture/18-law-of-leaky-abstractions.md) - All non-trivial abstractions are leaky to some degree
- [19 Tesler's Law](./3-architecture/19-teslers-law.md) - Every application has irreducible complexity that can only be shifted
- [20 CAP Theorem](./3-architecture/20-cap-theorem.md) - Distributed systems guarantee only two of: consistency, availability, partition tolerance
- [21 Second-System Effect](./3-architecture/21-second-system-effect.md) - Small, successful systems spawn overengineered, bloated replacements
- [22 Fallacies of Distributed Computing](./3-architecture/22-fallacies-of-distributed-computing.md) - Eight false assumptions new distributed system designers commonly make
- [23 Law of Unintended Consequences](./3-architecture/23-law-of-unintended-consequences.md) - Changing complex systems produces unpredictable surprises
- [24 Zawinski's Law](./3-architecture/24-zawinskis-law.md) - Every program expands until it can read mail

### 4. Quality & Testing (12 laws)
Principles related to code quality, testing, and bugs.

- [25 The Boy Scout Rule](./4-quality/25-boy-scout-rule.md) - Leave code better than you found it
- [26 Murphy's Law](./4-quality/26-murphys-law.md) - Anything that can go wrong will go wrong
- [27 Postel's Law](./4-quality/27-postels-law.md) - Be conservative in what you send; liberal in what you accept
- [28 Broken Windows Theory](./4-quality/28-broken-windows-theory.md) - Don't leave bad designs, wrong decisions, or poor code unrepaired
- [29 Technical Debt](./4-quality/29-technical-debt.md) - Technical debt comprises everything slowing software development
- [30 Linus's Law](./4-quality/30-linuss-law.md) - Given enough eyeballs, all bugs become shallow
- [31 Kernighan's Law](./4-quality/31-kernighans-law.md) - Debugging is twice as hard as writing code initially
- [32 Testing Pyramid](./4-quality/32-testing-pyramid.md) - Projects should have many unit tests, fewer integration tests, few UI tests
- [33 Pesticide Paradox](./4-quality/33-pesticide-paradox.md) - Repeatedly running same tests becomes less effective over time
- [34 Lehman's Laws](./4-quality/34-lehmans-laws.md) - Software reflecting reality must evolve with predictable evolution limits
- [35 Sturgeon's Law](./4-quality/35-sturgeons-law.md) - 90% of everything is crap
- [36 YAGNI](./4-quality/36-yagni.md) - Don't add functionality until it becomes necessary

### 5. Scalability (3 laws)
Principles related to performance, parallelization, and network effects.

- [37 Amdahl's Law](./5-scalability/37-amdahls-law.md) - Parallelization speedup is limited by non-parallelizable work fraction
- [38 Gustafson's Law](./5-scalability/38-gustafsons-law.md) - Significant parallelization speedup achievable by increasing problem size
- [39 Metcalfe's Law](./5-scalability/39-metcalfes-law.md) - Network value is proportional to the square of user numbers

### 6. Design Principles (5 laws)
Core principles of software design.

- [40 DRY](./6-design/40-dry.md) - Every knowledge piece must have single, unambiguous authoritative representation
- [41 KISS](./6-design/41-kiss.md) - Designs and systems should be as simple as possible
- [42 SOLID Principles](./6-design/42-solid-principles.md) - Five guidelines enhancing software design, maintainability, and scalability
- [43 Law of Demeter](./6-design/43-law-of-demeter.md) - Objects should interact only with immediate friends, not strangers
- [44 Principle of Least Astonishment](./6-design/44-principle-of-least-astonishment.md) - Software should behave least surprising to users and developers

### 7. Decision Making (12 laws)
Principles related to problem solving, judgment, and decision-making.

- [45 Dunning-Kruger Effect](./7-decisions/45-dunning-kruger-effect.md) - The less you know about something, the more confident you tend to be
- [46 Hanlon's Razor](./7-decisions/46-hanlons-razor.md) - Never attribute to malice what stupidity or carelessness adequately explains
- [47 Occam's Razor](./7-decisions/47-occams-razor.md) - The simplest explanation is often the most accurate one
- [48 Sunk Cost Fallacy](./7-decisions/48-sunk-cost-fallacy.md) - Sticking with choices due to prior investment, despite better alternatives
- [49 The Map Is Not the Territory](./7-decisions/49-map-is-not-territory.md) - Our reality representations aren't identical to reality itself
- [50 Confirmation Bias](./7-decisions/50-confirmation-bias.md) - Tendency favoring information supporting existing beliefs
- [51 The Hype Cycle & Amara's Law](./7-decisions/51-hype-cycle-and-amaras-law.md) - Overestimate short-term effects, underestimate long-term impact
- [52 The Lindy Effect](./7-decisions/52-lindy-effect.md) - Longer something's been in use, more likely it continues being used
- [53 First Principles Thinking](./7-decisions/53-first-principles-thinking.md) - Break complex problems into basic components, build up from there
- [54 Inversion](./7-decisions/54-inversion.md) - Solve problems by considering opposite outcomes, working backward
- [55 Pareto Principle](./7-decisions/55-pareto-principle.md) - 80% of problems result from 20% of causes
- [56 Cunningham's Law](./7-decisions/56-cunninghams-law.md) - Best way to get correct answers: post wrong answers, not questions

## 📊 Quick Reference

### By Category
| Category | Count | Focus |
|----------|-------|-------|
| Teams | 9 | Team composition, organizational structure |
| Planning & Estimation | 6 | Scheduling, estimation, metrics |
| Architecture | 9 | Design patterns, distributed systems |
| Quality & Testing | 12 | Code quality, testing, bug prevention |
| Scalability | 3 | Performance, parallelization |
| Design Principles | 5 | Core design principles |
| Decision Making | 12 | Problem-solving, judgment, bias |
| **Total** | **56** | |

## 📚 How to Use

### By Problem
```
"Why do projects keep falling behind on schedule?"
→ See Hofstadter's Law, The Ninety-Ninety Rule, Parkinson's Law

"How do we prevent code quality from degrading?"
→ See Broken Windows Theory, Technical Debt, The Boy Scout Rule

"Should we move to microservices?"
→ See CAP Theorem, Hyrum's Law, Law of Unintended Consequences

"Our API changes broke existing clients"
→ See Hyrum's Law, The Law of Leaky Abstractions

"Why is debugging so hard?"
→ See Kernighan's Law, Lehman's Laws
```

### Learning Path
1. **Beginners**: Start with Design Principles (DRY, KISS, SOLID) → Team Organization
2. **Developers**: Quality & Testing → Architecture → Planning & Estimation
3. **Leaders**: Teams → Decision Making → Architecture
4. **Architects**: Architecture → Design Principles → Scalability

## 💡 Key Insights

### Recurring Themes
1. **Irreducible Complexity**: Gall's Law, Tesler's Law, Zawinski's Law
2. **Predictability Limits**: Hofstadter's Law, Law of Unintended Consequences
3. **Trade-offs**: CAP Theorem, Tesler's Law
4. **Incremental Evolution**: Gall's Law, Second-System Effect
5. **Human Limitations**: Dunning-Kruger Effect, Confirmation Bias

## 🔗 External References

- Academic Papers: Referenced within each law's documentation
- Further Reading: Included in each law's reference section

---

**Last Updated**: April 22, 2026
**Total Laws**: 56
**Language**: English
