# Introduce Parameter Object

## Motivation

Introduce Parameter Object replaces a group of parameters that always appear together with a single object that carries them as fields. When the same cluster of arguments, such as start date and end date, or latitude and longitude, recurs across multiple method signatures, the repetition is itself a form of duplication. Beyond aesthetics, it creates maintenance risk: adding a new member to the cluster forces every signature and every call site to change in lockstep.

The technique gains additional leverage from the opportunity to relocate behavior. Once parameters live inside a dedicated class, validation logic, derived calculations, and formatting operations that once cluttered the calling methods can move onto the parameter object itself. A date range object, for instance, can answer whether a date falls inside it, eliminating scattered boundary checks from every consumer.

## Mechanics

1. Define a new class whose fields correspond to the grouped parameters. Default to immutability by accepting values through the constructor and exposing only read access.
2. Add the new object as a parameter to the target method using the mechanics of Add Parameter, passing the constructed object from each call site while still forwarding the original individual values internally.
3. One at a time, replace references to individual old parameters with field accesses on the new object. Compile and test after each replacement to isolate any mismatches.
4. Once all old parameters are gone from the signature, scan for helper methods or validation blocks that operate on the grouped data and consider relocating them onto the new class via Move Method or Extract Method.

## Indications

**Signs suggesting this refactoring:**
- The same subset of parameters appears in three or more method signatures across a module.
- A Data Clump is visible: developers find themselves copying and pasting identical argument lists when invoking related methods.
- Validation or transformation logic for a group of arguments is duplicated across multiple call sites or method bodies.

**When to avoid:**
- The grouped values have no semantic relationship beyond coincidentally appearing together in one method.
- The new class would contain only data with no behavior, producing a Data Class smell. If no operations naturally belong on the object, the grouping may be premature.
- The method signature is already stable and unlikely to gain additional parameters.

## Trade-offs

The most significant benefit is consolidation. A single typed object is easier to reason about than four or five primitive arguments, and it gives the compiler a named entity to validate at call sites. Future extensions require changes only to the parameter object's class rather than every method signature. The risk is over-engineering: if the parameter group has no natural cohesion or no methods to host on the new class, the result is a hollow data holder that adds indirection without behavior. The technique also introduces a new type into the codebase, which carries a naming and organization burden. When the cluster is small and stable, the overhead of the new class may outweigh the clarity gain.

## Connections

Introduce Parameter Object is closely related to Preserve Whole Object, which passes an already-existing object instead of decomposing it into primitives. Both address Long Parameter List, Data Clumps, and Primitive Obsession. The technique often enables further Extract Method opportunities because the new object provides a natural home for behavior that was previously scattered. Parameterize Method can complement it when the grouped parameters include a behavioral selector that would be better expressed as distinct methods. On the design pattern side, the resulting parameter object frequently evolves into a Value Object when immutability and equality semantics are added.

---

*Based on: Refactoring (Fowler, 1999)*
