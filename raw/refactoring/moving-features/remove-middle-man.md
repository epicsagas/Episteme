# Remove Middle Man

## Motivation

Remove Middle Man strips away a server class that has devolved into a pure pass-through, exposing the delegate directly to clients who need it. The degradation typically begins with Hide Delegate: a well-intentioned refactoring that wraps a few delegate calls behind server methods to reduce coupling. Over time, every new capability on the delegate demands a corresponding delegating method on the server. The server's method list becomes a mirror of the delegate's interface, each method a one-line forward with no added logic. At this point, the server class is not encapsulating anything — it is merely adding a hop to every call.

The Middle Man smell identifies this condition: a class whose methods consist entirely of delegation, with no validation, transformation, access control, caching, or domain logic of its own. Clients are already coupled to the delegate's semantics because the delegating methods expose the same parameters and return types. The indirection provides no protection against delegate interface changes — if the delegate renames a method, the server's wrapper must change too, and so must every caller. The layer has become pure overhead.

## Mechanics

1. Expose the delegate through a getter on the server class. If a getter already exists but was restricted in visibility, widen it to public. This provides clients with a direct route to the delegate object.

2. Replace each client call to a server delegating method with a direct call to the delegate. Update one call site at a time, replacing `server.doSomething()` with `server.getDelegate().doSomething()`. Test after each replacement.

3. After all clients access the delegate directly, remove the delegating methods from the server. If the server has no other methods beyond delegation, consider whether the server class itself should be removed entirely via Inline Class.

4. Evaluate whether the server's getter for the delegate is still needed. If clients now hold their own reference to the delegate through construction or injection, the getter may be removable as well.

## Indications

**Signs suggesting this refactoring:**
- Middle Man smell: the server class contains methods that do nothing but forward calls to a delegate
- Adding a feature to the delegate forces a rote copy of the method signature onto the server — the server's interface tracks the delegate's interface with no divergence
- Developers regularly bypass the server to access the delegate directly because the indirection adds friction without benefit
- The server performs no pre-processing, post-processing, logging, caching, or access control around its delegations

**When to avoid:**
- The server adds genuine value around delegation: validation, lazy initialization, access restrictions, audit logging, or result transformation
- The server encapsulates a volatile delegate that may be swapped for a different implementation — the indirection protects clients from that swap
- Exposing the delegate would create a wider dependency surface: clients would begin depending on the delegate's full interface rather than the narrowed view the server provides

## Trade-offs

Remove Middle Man is a simplification that trades encapsulation for directness. When the middle layer provides no encapsulation value — when it forwards without adding anything — the trade is unambiguously beneficial. Call paths become shorter, the class count drops, and developers spend less time tracing calls through hollow wrappers.

The risk is removing a layer that was quietly performing a service. Even a thin server may serve as a seam for future modification: adding logging, switching implementations, or enforcing invariants. Removing it locks clients into direct dependency on the delegate, and re-inserting the layer later requires touching every call site. The decision hinges on whether the server is a genuine abstraction boundary or a hollow shell. If it has any logic at all — even a null check or a default return — it is earning its place and should be retained. If its methods are literally one-line forwards with no branching, it is a candidate for removal.

## Connections

Remove Middle Man is the direct inverse of Hide Delegate — the two refactorings sit at opposite ends of a spectrum, and a class may oscillate between them as its responsibilities evolve. The refactoring targets the Middle Man smell by definition. It often triggers Inline Class when the server, stripped of its delegating methods, has nothing left. Move Method may follow: if some of the server's methods contained logic worth keeping, that logic should be relocated to the delegate or to another class before the server is eliminated. The Law of Demeter argues against Remove Middle Man in principle, but the law is a guideline, not an absolute — when the middle man enforces nothing, the law's benefit is theoretical while its cost in complexity is real.

---

*Based on: Refactoring (Fowler, 1999)*
