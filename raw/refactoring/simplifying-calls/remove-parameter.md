# Remove Parameter

## Motivation

Remove Parameter deletes an argument that a method no longer uses in its body. Unused parameters are not merely cosmetic noise; they impose a cognitive tax on every developer who reads the call site and wonders what the argument contributes. Each dead parameter also forces every caller to supply a value, which may mean computing or fetching something that is ultimately discarded. Over time, speculative parameters accumulate when developers add arguments "just in case" they become useful later, a habit that feeds the Speculative Generality smell.

Removing parameters keeps method signatures honest. When a signature lists only the arguments the body actually needs, the contract between caller and callee is transparent. This transparency accelerates onboarding, reduces the chance of passing the wrong value to a parameter that is silently ignored, and makes the method easier to test because each test needs fewer fixture values.

## Mechanics

1. Inspect the full inheritance chain. If the method overrides a superclass declaration or is itself overridden by a subclass, confirm that the parameter is unused across every variant. Polymorphic contracts require parameter-list consistency, so a single variant that still needs the argument blocks this refactoring.
2. Create a new version of the method without the target parameter. Forward calls from the old method to the new one, silently dropping the argument in the delegation.
3. Migrate every call site to the new signature. Remove the argument from each invocation. Compile and run tests after each batch to surface any remaining references.
4. Delete the old method, or deprecate it if it belongs to a public API that external clients depend on.

## Indications

**Signs suggesting this refactoring:**
- An IDE or static analyzer flags a parameter as unused within the method body.
- The parameter was added speculatively for a feature that was never implemented or was later removed.
- A recent round of Extract Method or Replace Parameter with Method Call has made the parameter's value available through another channel within the method.

**When to avoid:**
- The parameter is required by a superclass or interface contract, even if this particular implementation ignores it.
- The method is part of a callback or event-handler signature dictated by a framework, where removing the parameter would break the expected contract.
- The parameter exists for binary compatibility in a published library and cannot be removed without a major version bump.

## Trade-offs

The immediate gain is a signature that accurately reflects the method's dependencies. Callers become simpler because they prepare fewer arguments, and the method body becomes easier to reason about because every listed parameter has a visible role. The risk is minimal when the parameter is genuinely unused across the hierarchy. The primary caution involves inheritance: removing a parameter from one override while leaving it in another fragments the polymorphic contract and forces callers to know which concrete class they are invoking, undermining abstraction. In public APIs, removing a parameter is a breaking change, so the cost of migration must be weighed against the clarity benefit.

## Connections

Remove Parameter is the direct inverse of Add Parameter. It commonly follows Replace Parameter with Method Call, where a parameter's value becomes obtainable inside the method through a direct query, rendering the external argument redundant. The technique addresses the Speculative Generality smell by stripping arguments that were added for hypothetical future use. Rename Method often accompanies it because a slimmer signature may warrant a verb that better describes the reduced input set. When several unused parameters accumulate simultaneously, the refactoring pairs well with Introduce Parameter Object to first consolidate and then simplify the signature rather than pruning individual arguments one by one.

---

*Based on: Refactoring (Fowler, 1999)*
