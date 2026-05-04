# Replace Parameter with Method Call

## Motivation

Replace Parameter with Method Call removes an argument from a method's signature by having the method obtain the value itself through a direct query, eliminating the need for the caller to fetch and pass it. The telltale pattern is a call site that invokes a getter or computation, stores the result in a local variable, and immediately feeds it as an argument to another method on the same object or a closely related one. The intermediate handoff is redundant if the receiving method has sufficient context to perform the same query internally.

This refactoring reduces parameter lists, simplifies call sites, and localizes the data-dependency inside the method that actually uses it. It is particularly effective when the parameter value is derived from state that the method can already reach, such as fields on the same instance or services injected into the class. By removing the argument, the method becomes more self-contained and its signature shrinks, which counters the Long Parameter List smell.

## Mechanics

1. Confirm that the value passed as the parameter can be obtained from within the method without relying on data that is only available at the call site. If the value depends on a local variable or loop index unique to the caller, the parameter is genuinely necessary and this refactoring does not apply.
2. If the value-computation logic is complex or duplicated, apply Extract Method to isolate it into a dedicated query method that the target method can call.
3. Inside the target method, replace every reference to the parameter with a direct call to the query that produces the value.
4. Apply Remove Parameter to delete the now-unused argument from the signature and update all call sites accordingly.

## Indications

**Signs suggesting this refactoring:**
- Every call site computes the argument value in the same way before passing it, indicating the computation could be centralized inside the method.
- The argument is derived from fields or services that the receiving method can already access.
- A parameter was added speculatively to keep a future option open, but no caller currently varies the value.

**When to avoid:**
- The argument value depends on context that only the caller possesses, such as a loop iteration variable, a user input, or a value computed from a different data source.
- Different callers pass genuinely different values for the parameter, and moving the computation inside the method would force the method to decide which value to use, potentially introducing unwanted coupling.
- The method is a pure function or utility that deliberately operates only on its inputs to maintain testability and independence from object state.

## Trade-offs

The method becomes more self-sufficient and its signature shrinks, which improves readability at call sites and reduces the amount of data that callers must prepare. The technique also centralizes the query logic, eliminating duplicated computation across multiple call sites. The downside is increased coupling between the method and the source of the value. Where the method previously accepted any value the caller chose to provide, it now binds itself to a specific query path. This can reduce flexibility when future callers need to supply a different value from a different source. It can also make the method harder to test in isolation because the test must set up the state that the internal query relies on, rather than simply passing a value as an argument. In performance-sensitive contexts, if the query is expensive and some callers already have the result cached, forcing every invocation to recompute it inside the method is a regression.

## Connections

Replace Parameter with Method Call is the inverse of Add Parameter. It commonly follows Extract Method, which creates the query that the method then calls directly. The technique enables Remove Parameter as a subsequent step. Preserve Whole Object approaches the same problem from the opposite direction: rather than removing a parameter by internalizing the query, it replaces several parameters with a single object reference that the method queries internally. Replace Parameter with Explicit Methods is a related but distinct refactoring that eliminates a dispatch parameter by splitting the method rather than internalizing the value lookup. The technique supports the Tell, Don't Ask principle by moving decision logic into the object that owns the data rather than requiring callers to fetch and forward it.

---

*Based on: Refactoring (Fowler, 1999)*
