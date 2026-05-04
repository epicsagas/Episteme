# Separate Query from Modifier

## Motivation

Separate Query from Modifier splits a method that both returns data and changes object state into two distinct methods: a pure query that reads and returns a value without side effects, and a command that performs the state change without returning a result. The Command-Query Separation principle, articulated by Bertrand Meyer, underpins this technique: a method should either answer a question or perform an action, but never both. When a method violates this principle, callers cannot safely retrieve information without risking unintended mutations, and they cannot safely perform actions without having to discard an unwanted return value.

The problem becomes acute in multi-threaded or event-driven systems where a developer calls what appears to be a harmless getter and triggers a state transition that another component did not anticipate. It also complicates testing because asserting on the return value and asserting on the side effect must be done simultaneously, and the test cannot isolate one concern from the other.

## Mechanics

1. Create a new query method that returns the same value the original method does, but performs no state modification. Copy the read-only portion of the original method's logic into this new method.
2. Modify the original method to call the new query method and return its result, preserving the modification logic. At this stage, behavior is unchanged because the original method still does both things.
3. Find every call site that invokes the original method solely for its return value. Replace those calls with direct invocations of the new query method, removing any unintended side effects from those code paths.
4. Find every call site that invokes the original method for its side effect and ignores the return value. Remove the return-value usage and simplify the call to a standalone command.
5. Strip the return type from the original method, converting it into a pure command. Run tests after each step to confirm that no caller is broken by the separation.

## Indications

**Signs suggesting this refactoring:**
- A method name starts with `get` or `find` but also modifies internal state, logging, or external resources.
- Callers invoke the method and discard its return value, indicating they want only the side effect, or they invoke it and ignore the side effect because they want only the data.
- A test that calls the method to assert on the return value must also undo state changes in its teardown, signaling that the method's dual role is creating friction.

**When to avoid:**
- The method performs an atomic operation where the return value is an intrinsic part of the action, such as `pop` on a stack, which removes and returns the top element as an indivisible operation.
- Separating the query and command would introduce a race condition in concurrent code: by the time the command executes, the value returned by the query may have changed.
- The side effect is purely internal caching that has no visible impact on the object's externally observable state. Private caching inside a query is generally benign and does not warrant separation.

## Trade-offs

Clear separation lets callers retrieve data without fear of triggering mutations, and perform actions without managing unwanted return values. Methods with a single responsibility are easier to name, test, and reason about. The command method can evolve its mutation logic independently of the query method's read logic. The cost is an additional method on the class's API and, in some cases, an additional method call at each call site where both the query result and the state change are needed. In concurrent scenarios, splitting the two operations creates a window for race conditions unless the caller locks or otherwise coordinates the sequence. When atomicity is required, the two-method approach may need to be supplemented with a transaction or synchronization mechanism, which adds complexity that the original single method avoided by performing both operations in one step.

## Connections

Separate Query from Modifier is a direct implementation of the Command-Query Separation principle and the broader CQRS pattern at the method level. It pairs naturally with Replace Error Code with Exception because pure commands can signal failure through exceptions while pure queries simply return data. Replace Temp with Query often precedes it, creating the query method that this refactoring then isolates from side effects. The technique addresses side-effect-related aspects of the Long Method smell and helps clarify the intent of methods affected by the Side Effects smell. On the design-pattern side, it supports the Observer pattern by ensuring that notification methods (commands) do not accidentally return state that observers could depend on, which would create hidden coupling between the subject and its observers.

---

*Based on: Refactoring (Fowler, 1999)*
