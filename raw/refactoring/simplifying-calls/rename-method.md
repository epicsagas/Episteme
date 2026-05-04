# Rename Method

## Motivation

Rename Method replaces an unclear or misleading method name with one that accurately communicates the method's purpose. A poor name is not a cosmetic flaw; it is a latent defect. Every developer who encounters `process()`, `handle()`, or `calc()` must read the body to understand what happens, and every such reading wastes time and introduces the risk of misunderstanding. The name is the first and most visible contract between the method and its callers, and when that contract lies, the entire module suffers.

Names degrade over time as functionality evolves. A method originally named `getFileName` might later append a default extension, making `resolveFileName` a more honest description. Methods created under deadline pressure often receive shorthand names that made sense to the author but not to subsequent maintainers. Rename Method is the lowest-cost, highest-impact refactoring available: it changes no logic, yet it can dramatically improve how quickly a codebase is understood.

## Mechanics

1. Inspect the inheritance hierarchy for the method. If it is declared in a superclass or overridden in subclasses, the new name must propagate to every variant to preserve the polymorphic contract.
2. Create a new method with the improved name and copy the existing body into it. Modify the old method to delegate to the new one so that existing callers continue to work during the transition.
3. Migrate every call site to the new name. Use the IDE's rename refactoring if available, or perform a project-wide search and replace with careful review of each match to avoid false positives with similarly named identifiers.
4. Delete the old delegation method once all references are updated. If the method is part of a published API, mark it deprecated with a clear migration note instead of removing it outright.

## Indications

**Signs suggesting this refactoring:**
- A method name is vague or generic, such as `process`, `handle`, or `execute`, and cannot be understood without reading the body.
- Comments on the method attempt to explain what the name should have conveyed.
- A method's behavior has drifted from its original name due to feature additions, creating a mismatch between label and action.

**When to avoid:**
- The method is defined by a framework, interface, or external contract that cannot be modified.
- The name is cryptic but universally understood within the team's domain jargon, and renaming would break an established convention.
- The method is part of a stable public API where renaming would force breaking changes on consumers without sufficient payoff.

## Trade-offs

A precise name makes code self-documenting and reduces the need for explanatory comments. It accelerates code reviews, onboarding, and debugging because the method's role is apparent at a glance. The cost is purely mechanical: finding and updating every reference. Modern IDEs automate this process for most languages, reducing the risk of missed call sites to near zero. The only real danger lies in published APIs, where a rename is a breaking change. In those contexts, deprecation with a clear timeline is the responsible path, allowing consumers to migrate at their own pace. In rare cases, a longer name can feel verbose, but clarity always trumps brevity when the two conflict.

## Connections

Rename Method frequently accompanies Add Parameter and Remove Parameter because a signature change often warrants a name that reflects the new responsibility. It directly addresses the Comments smell: when a comment exists solely to explain what a method does, the right fix is often a better name rather than better documentation. The technique helps resolve Alternative Classes with Different Interfaces by standardizing method names across classes that perform the same role. Rename Method also supports Extract Method, where a newly extracted block needs a descriptive name from the start. When renaming constructors, consider whether Replace Constructor with Factory Method would simultaneously provide a more descriptive creation entry point.

---

*Based on: Refactoring (Fowler, 1999)*
