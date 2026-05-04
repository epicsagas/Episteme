# Preserve Whole Object

## Motivation

Preserve Whole Object replaces a sequence of individual values extracted from one object and passed as separate arguments with a single reference to the originating object itself. The hallmark pattern is a call site that chains multiple getter invocations, collects the results into local variables, and feeds them one by one into another method. A temperature-range checker might extract low and high values from a `TempRange` object only to pass them as two separate integers, when passing the `TempRange` directly would achieve the same result with far less glue code.

Beyond conciseness, the refactoring improves resilience to change. If the receiving method later needs a third datum from the same source, such as an average reading, the caller does not need to fetch and forward yet another value. The method simply reads it from the object it already holds. This makes the technique especially valuable when the source object is rich and the receiving method is likely to need more of its data over time.

## Mechanics

1. Add a parameter of the source object's type to the receiving method's signature, leveraging the Add Parameter mechanics.
2. Within the method body, replace references to each individual primitive parameter with the corresponding getter call on the new object parameter, one at a time. Test after each replacement.
3. Remove each replaced primitive parameter from the signature and update all call sites to drop the now-unnecessary getter chains.
4. After all individual parameters are eliminated, delete the getter-extraction code at the former call sites.

## Indications

**Signs suggesting this refactoring:**
- A caller extracts three or more values from the same object before passing them to a method.
- The same group of extracted values appears at multiple call sites, forming a Data Clump across invocation contexts.
- The receiving method's parameter list grows every time the source object gains a new field that the method needs.

**When to avoid:**
- Passing the whole object would create a dependency on a type that the receiving module has no legitimate reason to import, violating a layer boundary or increasing coupling.
- The source object is large and the method needs only one or two fields; the indirection of the whole object adds more complexity than it removes.
- The values come from different sources that merely happen to share a type, and passing the whole object would misrepresent the data's origin.

## Trade-offs

The principal advantage is a cleaner call site and a more adaptable receiving method. Future requirements for additional data from the source object require no changes at the caller, only within the method body. The technique also eliminates the Data Clumps and Primitive Obsession smells by replacing scattered primitives with a cohesive object reference. The main cost is tighter coupling: the receiving method now depends on the full type of the source object rather than on a minimal set of primitives. In layered architectures, this coupling can cross boundaries that were previously clean. A method in a service layer that once accepted only integers from a domain entity may now need to import that entity class directly. When the architectural boundary matters more than the parameter-list cleanliness, the trade-off favors keeping the primitives or introducing a lightweight Transfer Object.

## Connections

Preserve Whole Object is the natural companion to Introduce Parameter Object. Where Preserve Whole Object passes an existing object, Introduce Parameter Object creates a new one to bundle values from disparate sources. Both address Long Parameter List, Data Clumps, and Primitive Obsession. The technique frequently makes Replace Parameter with Method Call easier because the receiving method gains access to the source object's full interface and can invoke queries directly. It also relates to Hide Delegate in cases where the passed object exposes internals that the receiver then traverses further. When the source object is a Value Object, passing it whole is particularly natural because its immutability guarantees that the method cannot mutate the caller's state.

---

*Based on: Refactoring (Fowler, 1999)*
