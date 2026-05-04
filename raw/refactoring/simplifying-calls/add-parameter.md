# Add Parameter

## Motivation

Add Parameter extends a method's signature so it can receive data that was previously unavailable within its scope. This technique becomes necessary whenever evolving requirements force a method to operate on information it cannot reach through existing fields, properties, or other parameters. A common trigger is a business rule change: a discount calculator that once applied a flat rate now needs a customer-tier enum, or a logging function that must include a correlation identifier it never required before.

The alternative to adding a parameter is introducing a private field on the owning object, but fields imply persistent state. When the data is transient, request-scoped, or varies per invocation, passing it as a parameter communicates intent more accurately. The danger lies in overuse. Each new parameter increases the method's coupling to its callers and nudges the signature toward the Long Parameter List smell. Before reaching for this refactoring, consider whether Preserve Whole Object or Introduce Parameter Object would consolidate several scattered values into a single coherent argument.

## Mechanics

1. Search the entire inheritance hierarchy for overrides or super calls of the target method. Every declaration in that chain must receive the same treatment to preserve polymorphic contracts.
2. Create a new version of the method that includes the additional parameter in its signature, preserving the original body. Place a temporary call from the old method to the new one, passing a safe default such as null or zero for the added argument.
3. Migrate every call site to the new signature, supplying a meaningful value for the extra argument. Compile and run tests after each batch of changes to catch signature mismatches early.
4. Remove the old method entirely, or mark it deprecated if it belongs to a published API that external consumers depend on.

## Indications

**Signs suggesting this refactoring:**
- A method body contains a comment explaining what extra information it wishes it had access to.
- Conditional branches inside a method handle cases that differ only by a single datum the caller already possesses.
- The same value is being fetched indirectly through global state or service locators when the caller could pass it directly.

**When to avoid:**
- The missing data is inherently part of the object's identity and should live as a field.
- Several parameters accumulate over time, signaling that Preserve Whole Object or Introduce Parameter Object is the more appropriate move.
- The method already exhibits a Long Parameter List; adding another argument deepens the smell rather than solving the real design issue.

## Trade-offs

On the positive side, the refactoring is mechanically straightforward and keeps the change local to one method's contract. It avoids inflating object state with data that has no business persisting beyond a single call. However, each new parameter tightens the coupling between caller and callee, and if multiple callers must supply the same derived value, duplication creeps in. Repeated application of Add Parameter without consolidation is a leading cause of bloated signatures, which themselves become a barrier to readability and testing. When three or more related parameters accumulate, the trade-off tips in favor of grouping them through Introduce Parameter Object.

## Connections

This refactoring is the inverse of Remove Parameter. It frequently serves as a stepping stone toward Introduce Parameter Object, where a cluster of individually added arguments gets bundled into a single structure. Preserve Whole Object addresses a similar data-availability gap but resolves it by passing an existing object rather than introducing a new argument. Rename Method often accompanies Add Parameter because the expanded responsibility may warrant a more descriptive verb phrase. The technique can also expose the Data Clumps smell: if the same set of parameters appears across multiple signatures, that repetition signals a missing abstraction.

---

*Based on: Refactoring (Fowler, 1999)*
