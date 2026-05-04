# Split Temporary Variable

## Motivation
Split Temporary Variable addresses the confusion that arises when a single local variable is repurposed to hold unrelated values at different points in a method. A developer reading `temp = height * width` on line 10 and then `temp = 2 * (height + width)` on line 20 must mentally reset their understanding of what `temp` represents at each stage, and any modification to one computation risks inadvertently affecting the other. Named variables should encode meaning, and a variable that means different things at different times fails at that job. Introducing a distinct variable for each purpose makes every assignment self-documenting and every reference unambiguous.

This problem frequently appears in methods that perform sequential calculations — computing perimeter and area, accumulating subtotals and then tax, or staging data through multiple transformations. The original developer chose a generic name out of haste or habit, and subsequent maintainers perpetuated the pattern rather than introducing properly named variables. The refactoring is straightforward but its impact on readability is disproportionate to its cost.

## Mechanics
1. Identify a local variable that receives more than one assignment within the method, particularly when the assignments compute conceptually different values.
2. At the first assignment, rename the variable to a name that reflects the specific value being computed — for example, `perimeter` instead of `temp`.
3. Update every reference between the first assignment and the second assignment to use the new name, being careful not to change references that occur after the second assignment.
4. At the second assignment, introduce a new variable with its own descriptive name — for example, `area` — and update all references from that point to the next assignment (or the end of the method) to use this second name.
5. Repeat for each subsequent assignment to the original variable, giving each span its own distinctly named variable.
6. Mark each new variable as `final` or `const` where the language supports it, since each variable now receives exactly one assignment, and run tests to confirm correctness.

## Indications

**Signs suggesting this refactoring:**
- A local variable with a generic name like `temp`, `result`, `value`, or `x` receives multiple assignments that compute unrelated quantities
- A developer must scroll between an assignment and its usage to confirm what the variable holds at that point in the method
- A variable changes type or semantic meaning between assignments — for example, holding a raw count and then a formatted string

**When to avoid:**
- The variable is used as a deliberate accumulator where each assignment adds to a running total or appends to a collection — this is a single evolving purpose, not multiple unrelated purposes
- The method is short enough that the variable's repurposing is visible on a single screen and the assignments are adjacent, making the mental reset trivial
- The language does not support local variable shadowing or redeclaration, and introducing new names would require restructuring the control flow in ways that reduce clarity

## Trade-offs
Splitting a multi-purpose variable into single-purpose variables yields immediate readability gains: each name describes its value, the compiler can flag accidental reassignment through `final` or `const`, and future modifications to one computation cannot inadvertently break another. The refactoring also prepares the method for Extract Method, because each single-assignment variable can be cleanly passed as a parameter or returned as a result without tracking multiple lifetimes. The cost is minimal — a few extra variable declarations — and is universally worth paying unless the method is so short that the split introduces more lines than the original. The one subtlety to watch for is control flow: if assignments occur inside conditional branches and the variable is read after the branch, splitting may require careful analysis to ensure each path assigns the correct variable.

## Connections
Split Temporary Variable is a prerequisite for Replace Temp with Query, which requires the variable to have exactly one assignment before the expression can be extracted into a query method. It also unblocks Extract Method by ensuring that each variable in the method has a clear single responsibility, making it straightforward to decide which variables become parameters and which become return values. The refactoring addresses a specific variant of the Temporary Field smell at the local level and encourages the Single Responsibility Principle at the variable scope. When the split reveals that two computations share no dependencies and could live in separate methods entirely, the developer has discovered an opportunity for deeper structural decomposition.

---

*Based on: Refactoring (Fowler, 1999)*
