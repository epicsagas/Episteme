# Extract Subclass

## Motivation

Extract Subclass is a refactoring technique that moves specialized behavior from a general-purpose class into a dedicated child class, isolating optional or conditional logic that only applies in certain contexts. Classes tend to accumulate features over time -- a method guarded by a boolean flag, a field populated only for premium users, a branching path inside an otherwise straightforward algorithm. These additions make the class harder to read because every method must consider cases that most instances will never encounter.

By pushing that specialized logic into a subclass, the parent class reverts to a clean, general implementation while the child carries the extra weight only when instantiated. This separation improves readability: developers working with the base case never see code paths irrelevant to their work.

## Mechanics

1. Identify the fields and methods that serve a specialized case. Look for conditional checks on a discriminator field, feature flags, or null-guarded optional data as primary candidates.
2. Create a new class that extends the original. Add a constructor that forwards required parameters to the parent.
3. Search all construction sites and replace those that need the specialized behavior with instantiation of the new subclass.
4. Move the identified methods and fields from the parent to the child using Push Down Method and Push Down Field. Start with methods, since fields often support method logic and their relocation becomes obvious once methods have moved.
5. Remove any discriminator fields and conditional branches from the parent that existed solely to select between general and specialized behavior. Replace them with polymorphic dispatch: the parent declares an abstract or default method, and each subclass provides its own implementation.
6. Run tests after each relocation to confirm that call sites still resolve correctly.

## Indications

**Signs suggesting this refactoring:**
- A class contains conditional branches that select behavior based on a type code or feature flag that never changes after construction.
- Certain fields are always null or empty for a known subset of instances, indicating they belong to a separate variant.
- Methods contain large `if` blocks that could become the sole implementation of a subclass method.

**When to avoid:**
- The class needs to vary along multiple independent dimensions simultaneously. Inheritance is single-axis: a `Vehicle` subclassed by `LandVehicle` and `WaterVehicle` cannot also express combinations like `AmphibiousVehicle` without diamond-hierarchy problems. In that situation, composition using Extract Class or the Strategy pattern is a better fit.
- The specialized behavior is small enough that extracting it adds more structural overhead than the conditional it replaces.

## Trade-offs

The immediate benefit is a clearer mental model: each class tells a single story, and developers can reason about the general case without tripping over special-case branches. Polymorphic dispatch also eliminates the risk of forgetting to update a switch statement when a new variant appears. The cost is hierarchy proliferation. Every extracted subclass is another type to maintain, and if the system later needs to cross-cut those variants with an orthogonal concern, the inheritance tree may become a liability. Teams should weigh whether the specialization axis is stable and singular before committing to a subclass.

## Connections

Extract Subclass is the inverse of Collapse Hierarchy, which removes a subclass that has become indistinguishable from its parent. It frequently follows Extract Class in a refactoring sequence: first the optional features are gathered into a separate class, then if inheritance semantics make sense, the class becomes a subclass of the original. Push Down Method and Push Down Field are the workhorse refactorings used during extraction. The technique addresses the Large Class smell by shedding optional responsibilities. It also relates to the Template Method pattern, where the parent defines an algorithm skeleton and extracted subclasses fill in the variable steps.

---

*Based on: Refactoring (Fowler, 1999)*
