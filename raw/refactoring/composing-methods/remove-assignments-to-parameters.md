# Remove Assignments to Parameters

## Motivation
Assigning a new value to a method parameter inside the method body creates a subtle ambiguity: anyone reading the code must track whether the parameter still holds its original argument value or has been overwritten at some point during execution. This confusion intensifies in languages that support pass-by-reference semantics, where the reassignment can mutate the caller's variable unexpectedly. Even in strictly pass-by-value languages, the pattern misleads developers who assume parameters are immutable contracts between caller and callee. Replacing parameter assignments with a dedicated local variable makes the data flow explicit and eliminates the ambiguity.

The discipline of treating parameters as read-only inputs also simplifies reasoning about method behavior during code review and debugging. When a parameter is never reassigned, the reader knows its value is fixed for the entire method execution, which reduces the mental state that must be tracked. This convention aligns with the Command-Query Separation principle and makes the method easier to extract or move later, since the parameter list cleanly represents external dependencies.

## Mechanics
1. Locate every assignment to a parameter within the method body, including compound assignments like `param += delta` and `param ??= default`.
2. Declare a new local variable initialized to the parameter's value at the point where the first reassignment occurs, giving it a name that reflects its evolving role — for example, `result` for an accumulator or `remaining` for a countdown.
3. Replace all subsequent references to the parameter with references to the new local variable, ensuring the parameter itself is only read from that point forward.
4. Run tests to verify that the method's output is unchanged, paying particular attention to edge cases where the parameter was conditionally reassigned.

## Indications

**Signs suggesting this refactoring:**
- A method body contains an assignment statement whose left-hand side is a formal parameter, such as `inputVal -= 2` or `name = name.trim()`
- Code comments near the assignment attempt to explain whether the modification affects the caller, indicating that the parameter mutation is non-obvious
- A method uses the same parameter name to mean two different things — the original input and a derived intermediate value — depending on how far execution has progressed

**When to avoid:**
- The language idiom conventionally uses parameter reassignment for output parameters, and callers depend on this behavior — changing it would break the established contract
- The parameter is an output or reference parameter explicitly decorated with language-level annotations such as `out` or `ref` in C#, where reassignment is the intended mechanism

## Trade-offs
The clear benefit is reduced cognitive load during code reading: parameters become guaranteed-stable inputs, and all mutations flow through explicitly named local variables whose scope and purpose are easier to audit. This separation also prevents accidental side effects in pass-by-reference languages, where overwriting a parameter could silently corrupt data in the calling context. The cost is a small increase in local variable declarations, which is negligible compared to the clarity gained. In languages with aggressive optimizing compilers, introducing a local variable has zero runtime overhead because the compiler generates identical machine code for both forms. The primary situation where this refactoring might be unwarranted is in a short method where the parameter mutation is the entire method body and the intent is immediately obvious — but even then, using a local variable communicates the design more clearly to less experienced readers.

## Connections
Remove Assignments to Parameters is a prerequisite for Clean Code disciplines that treat method signatures as immutable contracts. It supports Extract Method by ensuring that parameters can be passed directly into the extracted method without risking unintended mutation of the original parameter. The refactoring helps address the Side Effects smell and complements Introduce Parameter Object by making parameter usage patterns more visible and easier to group. In object-oriented design, it aligns with the Command pattern's principle that input data should not be altered during command execution.

---

*Based on: Refactoring (Fowler, 1999)*
