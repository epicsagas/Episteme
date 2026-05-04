# Pull Up Method

## Motivation

Pull Up Method is a refactoring technique that moves a method from subclasses into their common superclass when the method's implementation is identical or can be made identical across all children. Duplicate methods across sibling classes are a maintenance hazard: a bug fix applied to one copy may be forgotten in another, and the drift between copies grows as developers make localized changes. Consolidating the method in the superclass creates a single source of truth for that behavior.

This technique also applies when subclasses override a superclass method with functionally equivalent implementations. The redundancy in that case is slightly different -- the method already exists in the parent -- but the remedy is the same: remove the overrides and let the parent's implementation serve all children.

## Mechanics

1. Compare the candidate methods across all subclasses. Look beyond surface formatting to verify that the logic, return types, and side effects are functionally equivalent. If they differ, refactor the implementations toward uniformity first -- this may require introducing parameter changes or extracting helper methods.
2. If the methods use different parameter names or counts, standardize the signature. All versions must accept the same arguments before they can be represented by a single method.
3. Check whether the method references any subclass-specific fields or methods. If it does, pull those dependencies up first using Pull Up Field, or declare abstract accessor methods in the superclass that each subclass implements.
4. Copy the method into the superclass. Ensure the visibility modifier is appropriate -- protected or public depending on who needs to call it.
5. Delete the method from each subclass.
6. Search client code for places where a subclass type was used as the declared type for the sole purpose of accessing this method. Replace those with the superclass type to widen polymorphic flexibility.
7. Run the full test suite.

## Indications

**Signs suggesting this refactoring:**
- Two or more subclasses contain methods with the same name, signature, and body.
- Subclasses override a parent method with identical implementations, suggesting the parent version was lost or never written.
- A bug fix in one subclass's method reveals that the same fix is needed in sibling subclasses but was overlooked.

**When to avoid:**
- The methods look similar but serve subtly different purposes in each subclass -- for instance, `calculateTotal` that rounds down in one variant and rounds up in another. Pulling up the method would erase a meaningful behavioral difference.
- Only one subclass currently holds the method, and generalizing it would require introducing abstract hooks in the superclass that no other child needs.

## Trade-offs

The primary gain is elimination of duplication: one method to read, one method to test, one method to fix. This directly reduces the risk of consistency bugs and lowers the cognitive load on anyone modifying the behavior. The superclass also becomes a more accurate representation of the shared concept, which improves the hierarchy's communicative value. The risk is over-generalization. If a future subclass needs a different implementation of the same method, developers may be tempted to override the pulled-up version, re-introducing the very duplication the refactoring removed. The technique is safest when the shared behavior is genuinely invariant across all present and foreseeable subclasses.

## Connections

Pull Up Method is the inverse of Push Down Method and is the central sub-operation in Extract Superclass. It frequently follows Pull Up Field, since methods often depend on fields that must be elevated first. The technique is a prerequisite for Form Template Method, which pulls up the shared steps of an algorithm while leaving variable steps abstract. It addresses the Duplicate Code smell across class boundaries and helps resolve the Parallel Inheritance Hierarchies smell when applied systematically across a set of sibling classes.

---

*Based on: Refactoring (Fowler, 1999)*
