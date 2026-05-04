# Push Down Method

## Motivation

Push Down Method is a refactoring technique that moves a method from a superclass into the specific subclasses that actually invoke it. A method that lives in a parent class but serves only one or two children misrepresents the hierarchy: developers reading the superclass expect every method there to be universally applicable, and those reading an unaffected subclass may be confused by inherited behavior they never call. Relocating the method to the subclasses that depend on it restores clarity at each tier.

The technique becomes relevant when features are pruned from a hierarchy or when an earlier generalization proves too aggressive. A method that once served all children may, after refactoring, be relevant to only a subset. Rather than leaving it in the parent as dead weight, pushing it down makes the superclass's contract honest.

## Mechanics

1. Identify which subclasses actually call the method or override it with meaningful behavior. If only a single subclass uses it, the relocation target is obvious. If two or three out of many need it, consider creating an intermediate subclass to hold the shared method and avoid duplication.
2. Copy the method implementation into each target subclass. Adjust the visibility as needed -- a method that was protected in the parent may become private in a subclass if no further children need access.
3. Remove the method from the superclass.
4. Search all call sites. Where the caller already holds a reference to the specific subclass, no change is needed. Where the caller holds a superclass reference, it must be narrowed to the subclass type, or the call must be restructured via polymorphism -- for example, by declaring an abstract version of the method in the parent that each relevant subclass overrides.
5. Run tests to confirm that every invocation still resolves correctly.

## Indications

**Signs suggesting this refactoring:**
- A superclass method is called only from within one or two subclass methods, while other children never reference it.
- Code review reveals that a parent class method exists solely to support a feature specific to certain child types.
- Adding a new subclass forces an override of a parent method that throws "unsupported operation," indicating the method does not belong in the general contract.

**When to avoid:**
- The method is called through superclass references in client code, and those callers cannot be narrowed to specific subclass types. Removing it from the parent would break the call sites.
- The effort to relocate the method exceeds the clarity gained, particularly when the method is small and its presence in the parent is harmless.

## Trade-offs

Pushing a method down makes the superclass's API a more accurate reflection of what all subclasses genuinely share. Developers can trust that anything declared in the parent applies universally, which reduces the need for defensive checks and "unsupported operation" overrides in child classes. The trade-off is that if multiple subclasses need the same implementation, the method body is duplicated unless an intermediate class is introduced. That intermediate class adds hierarchy depth, which itself carries a navigation cost. The technique is most beneficial when the method is clearly non-universal and the number of needing subclasses is small.

## Connections

Push Down Method is the inverse of Pull Up Method and is a common sub-operation within Extract Subclass -- as specialized behavior is isolated, methods that support it follow the data downward. It pairs naturally with Push Down Field since a method often depends on a field that should be relocated alongside it. The technique addresses the Refused Bequest smell by removing methods that a subclass inherits but never uses. It also relates to Replace Inheritance with Delegation when the pattern of refused methods suggests that the subclass should not inherit from the parent at all.

---

*Based on: Refactoring (Fowler, 1999)*
