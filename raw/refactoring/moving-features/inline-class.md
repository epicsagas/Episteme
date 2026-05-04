# Inline Class

## Motivation

Inline Class collapses a class that has been drained of responsibility into a host class that absorbs its remaining features. Classes become candidates for inlining when prior refactoring rounds have relocated their substantive behavior elsewhere, leaving behind a hollow shell with one or two trivial methods and a handful of fields. Maintaining a separate class for such minimal content imposes cognitive cost — developers must navigate two files, understand the relationship between them, and trace calls across the boundary — without any compensating clarity gain. The class no longer represents a distinct concept; it is an artifact of earlier decomposition that has since been superseded.

This situation also arises from Speculative Generality: a class created in anticipation of future variation that never materialized. The empty abstraction sits in the codebase, adding to the surface area that must be read, reviewed, and maintained without delivering any flexibility in return.

## Mechanics

1. Identify the target host class — the class that will absorb the donor's features. The host is typically the class that already holds the strongest dependency on the donor, either as a field reference or as the most frequent caller of its methods.

2. Add public methods and fields to the host that mirror the donor's public interface. Each new host method delegates to the donor instance, preserving behavior while establishing the forwarding layer.

3. Replace every external reference to the donor class with a reference to the host class. Update variable declarations, constructor calls, type annotations, and imports. Test after each batch of replacements, focusing on cases where the donor was used as a parameter type or return type.

4. Use Move Method to transfer each method from the donor directly into the host, replacing the delegating stub with the actual implementation. Apply Move Field to relocate the donor's fields. Work method by method, testing after each transfer.

5. Delete the donor class once it contains no remaining fields or methods. Remove its file, constructor, and any residual imports or factory methods that instantiated it.

6. Run the full test suite. Pay attention to serialization, equality checks, and reflection-based code that may reference the old class by name.

## Indications

**Signs suggesting this refactoring:**
- A class has only one or two methods and a small number of fields, none of which express a concept distinct from the host class
- Lazy Class appears: the class exists but contributes no meaningful abstraction
- Speculative Generality: the class was created for anticipated flexibility that was never exercised
- Shotgun Surgery results from changes that must touch both the thin class and its host in lockstep

**When to avoid:**
- The class, though small, represents a genuine domain concept with a clear identity that differs from any potential host
- The class serves as a shared dependency used by multiple unrelated clients — inlining it into one host would force the others to depend on that host
- The class participates in an inheritance hierarchy or implements an interface that other code references polymorphically
- Future plans genuinely call for the class to regain responsibilities, making its current thin state temporary

## Trade-offs

Inline Class reduces structural complexity at the cost of increasing the host class's size and responsibility. When the donor class carried no meaningful abstraction, the trade is unambiguously favorable: fewer files, fewer relationships, and less indirection with no loss of clarity. The host class becomes the single authority for the merged concept, and developers can understand the complete behavior in one place.

The risk emerges when the donor's identity, however thin, represents a boundary that prevented the host from growing too large. Inlining a class that filtered access, enforced invariants, or provided a named abstraction can cause the host to balloon and lose focus. If the host was already approaching the threshold of Large Class, absorbing more content may trigger the very problems Inline Class is meant to solve — just in the opposite direction. In such cases, the correct response may be to re-extract the behavior into a better-factored class rather than inlining into a struggling one.

## Connections

Inline Class is the direct inverse of Extract Class — applying one reverses the other. It is frequently composed with Move Method and Move Field as the mechanical means of transferring features before the donor is deleted. The refactoring directly addresses Lazy Class and Speculative Generality by eliminating classes that do not earn their place. It also mitigates Shotgun Surgery when a thin class forces changes to be coordinated across two tightly coupled files. Collapse Hierarchy serves a related purpose, merging a subclass into its parent when the inheritance distinction has lost meaning.

---

*Based on: Refactoring (Fowler, 1999)*
