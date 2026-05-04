# Hide Delegate

## Motivation

Hide Delegate eliminates the need for clients to understand the internal structure of the objects they use. When client code reaches through a server object to access a delegate — calling `manager.department().budget()` instead of `manager.getDepartmentBudget()` — it binds the client to the intermediate object graph. Any restructuring of that graph ripples outward into every caller, violating encapsulation at the architectural level. The client now depends not just on the server's interface but on the server's implementation: which objects it holds, how they connect, and what methods they expose.

This dependency manifests as Message Chains, where a single expression strings together multiple getter calls to reach a distant object. Each link in the chain represents a coupling point. Moving, renaming, or replacing any intermediate node forces changes in every client that traverses the chain. The Law of Demeter formalizes this concern: an object should interact only with its immediate collaborators, not with the collaborators of its collaborators.

## Mechanics

1. Identify a delegate method that clients access by reaching through the server. Look for call sites where the client retrieves an object from the server and then immediately invokes a method on that retrieved object.

2. Add a new method to the server class with the same signature as the delegate method. The server method's body simply forwards the call to the delegate. Name the method to reflect the server's perspective — not what the delegate does internally, but what action the server performs on behalf of the client.

3. Replace each client call site so it invokes the new server method instead of traversing the chain. Update one call site at a time and test after each replacement to catch subtle differences in behavior.

4. Once no client directly accesses the delegate through the server, remove or restrict the accessor that previously exposed it. If the accessor was a public getter, consider reducing its visibility or deleting it entirely.

5. Repeat for each additional delegate method that clients reach through the server. As the set of delegating methods grows, evaluate whether the server is becoming a Middle Man — if so, the refactoring has overshot its target and Remove Middle Man may be warranted.

## Indications

**Signs suggesting this refactoring:**
- Client code contains chains of method calls like `a.getB().getC().doSomething()` where each intermediate step is a structural detail, not a domain operation
- Inappropriate Intimacy appears: clients know too much about the internal composition of the classes they depend on
- Changes to the delegate class's interface force widespread edits in client code that has no direct reason to depend on it

**When to avoid:**
- The server would need to expose dozens of delegating methods, transforming it into a pure pass-through with no logic of its own — this creates a Middle Man smell
- Clients legitimately need full access to the delegate's capabilities for composition, transformation, or pipeline operations where hiding the intermediate object would obscure the intent
- The delegation chain is shallow (one hop) and stable, making the encapsulation benefit marginal relative to the added interface surface

## Trade-offs

Hide Delegate trades a small increase in server interface surface for a significant reduction in client-to-implementation coupling. Each added delegating method is a thin wrapper, but it establishes a stable contract: the server promises to fulfill this request, regardless of which internal object actually performs the work. When the underlying object graph changes — a field is renamed, a collaborator is replaced — only the server's delegating methods need updating; clients remain untouched.

The breaking point arrives when the server accumulates so many delegating methods that it ceases to have meaningful logic of its own. At that threshold, the server has become a Middle Man, adding a layer of indirection without corresponding encapsulation value. The judgment call is whether the client is genuinely insulated from change. If the delegate's interface is stable and the server adds no value beyond forwarding, the delegation layer is pure overhead. Conversely, if the server performs validation, caching, access control, or logging around the delegation, the wrapper earns its place in the design.

## Connections

Hide Delegate is the inverse of Remove Middle Man — the two refactorings move in opposite directions along the encapsulation spectrum. It directly targets the Message Chains smell by short-circuiting the chain at the server boundary. Inappropriate Intimacy also diminishes because clients no longer reach into the server's internal structure. The Law of Demeter provides the design rationale: hiding the delegate enforces the principle that objects should talk to their friends, not to strangers. Facade Pattern operates on the same principle at a larger scale, shielding clients from subsystem complexity behind a unified interface. Move Method is a related technique for relocating behavior wholesale rather than wrapping it with delegation.

---

*Based on: Refactoring (Fowler, 1999)*
