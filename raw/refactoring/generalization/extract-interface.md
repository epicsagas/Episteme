# Extract Interface

## Motivation

Extract Interface is a refactoring technique that distills a shared behavioral contract from one or more classes into a standalone abstract type. The need arises when different clients interact with the same subset of an object's capabilities, or when two unrelated classes happen to expose matching method signatures. By formalizing that common subset as an interface, the codebase gains a named role that clients can depend on without coupling to a concrete implementation.

This technique is particularly valuable when preparing a system for multiple implementations of the same responsibility. A class that parses documents might be the only implementation today, but declaring a `DocumentParser` interface early lets future contributors add alternative parsers without touching existing call sites. It also strengthens testability: test doubles can implement the interface instead of extending a concrete class.

## Mechanics

1. Identify the methods that clients actually invoke on the class or classes in question. Ignore the rest of the public API -- the interface should capture only the role being consumed.
2. Create a new interface declaring those methods with identical signatures.
3. Make the original class or classes implement the new interface. The compiler will confirm that every declared method exists with a compatible type.
4. Change client type hints, parameter declarations, and variable annotations from the concrete class to the interface. This step is where the real decoupling happens: clients now depend on an abstraction rather than a specific implementation.
5. Run the full test suite. Compilation errors at this point reveal clients that relied on methods outside the new interface, which may indicate a second interface is needed or that the role boundary should expand.

## Indications

**Signs suggesting this refactoring:**
- Multiple callers use the same narrow subset of a class's public methods, while ignoring the rest.
- Two or more unrelated classes share method signatures that serve a common purpose, hinting at an unnamed role in the domain.
- A concrete class appears in parameter lists and return types where only behavioral capability matters, not object identity.

**When to avoid:**
- The class already has substantial duplicate code in common with another class; Extract Interface captures contracts only, not behavior. In that scenario, Extract Superclass or Extract Class may be more appropriate.
- The interface would contain only a single method with no realistic prospect of additional implementations. A one-method interface can be useful for lambda compatibility, but creating one purely for speculative flexibility adds noise.

## Trade-offs

The primary gain is loose coupling: clients that depend on an interface can work with any future implementation, including mocks, stubs, and alternative strategies. This pays dividends in testing and in evolving the system without rippling changes through call sites. The cost is an additional type in the codebase, which increases the number of files developers must navigate. In a large system with hundreds of single-implementation interfaces, the signal-to-noise ratio can deteriorate. The technique also requires discipline around interface naming: a poorly named interface obscures intent more than no interface at all.

## Connections

Extract Interface works well in combination with Extract Superclass when both a contract and shared implementation are needed -- the interface defines the role while the superclass provides default behavior. It pairs naturally with the Strategy pattern, since strategies are defined by the interface they implement. Where Extract Interface adds an abstraction layer, Collapse Hierarchy removes one; the two occasionally appear in the same refactoring session when a hierarchy is being flattened and restructured into interface-based composition. The technique directly addresses the Divergent Change smell by isolating each client's view of a class into its own narrow contract.

---

*Based on: Refactoring (Fowler, 1999)*
