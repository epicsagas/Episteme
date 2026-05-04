# Extract Method

## Motivation
Extract Method addresses the most common readability problem in codebases: methods that have grown beyond a single screen of text or a single coherent purpose. When a method contains comment-separated blocks, handles multiple responsibilities, or requires careful reading to understand its flow, the code is signaling that it wants to be decomposed. Named method calls serve as documentation that the original comments were struggling to provide, and the resulting smaller methods become reusable units that other parts of the codebase can call directly.

This refactoring is often the first step in a larger cleanup effort because it creates the building blocks that other refactorings depend on. A team reviewing a legacy codebase will typically apply Extract Method dozens of times before touching any other technique. The resulting methods also improve debuggability: stack traces point to named operations rather than anonymous line numbers, and breakpoints can target specific behaviors precisely.

## Mechanics
1. Identify a contiguous block of statements within the source method that performs one distinguishable task, even if that task is only a few lines long.
2. Create a new method with a name that describes what the block does, not how it does it — prefer `applyVolumeDiscount` over `loopThroughItems`.
3. Copy the identified block into the new method body, then replace the original block with a call to the new method.
4. Audit local variables used inside the block: read-only variables become parameters, while variables computed for the first time become return values or new local declarations.
5. If the block modifies more than one variable that the caller needs afterward, consider applying Split Temporary Variable first or returning a small result object.
6. Run tests after each extraction. A single large extraction is riskier than several small ones, so prefer incremental steps.

## Indications

**Signs suggesting this refactoring:**
- A method spans more than roughly ten lines or requires scrolling to read in full
- Comment lines inside a method act as section headers separating logical blocks
- The same group of lines appears in two or more places within the codebase
- A reviewer asks "what does this section do?" because the intent is not obvious from the code itself

**When to avoid:**
- The extraction would produce a trivial one-line wrapper whose name adds no information beyond what the line already expresses
- The resulting method signature would require an excessive parameter list, indicating the logic is too tightly coupled to local state and needs Replace Method with Method Object instead

## Trade-offs
The primary gain is readability: a well-named method communicates intent at the call site without requiring the reader to dive into implementation details. Smaller methods are also easier to test in isolation and simpler to override in subclasses. The cost is an increase in the total number of methods, which can make the class's public surface harder to navigate if extractions are done without disciplined naming. Very thin extractions — where the new method merely wraps a single expression — add indirection without proportional clarity and are candidates for Inline Method in reverse. Performance overhead from additional call frames is negligible in modern runtimes and is an inappropriate reason to avoid this refactoring.

## Connections
Extract Method is the inverse of Inline Method and frequently precedes Replace Temp with Query, because extracted code often reveals temporary variables that should become queries. It directly addresses the Long Method and Duplicate Code smells, and can mitigate Comments by turning explanatory comments into self-documenting method names. The technique enables higher-order refactorings such as Form Template Method and Introduce Parameter Object by establishing the discrete operations those patterns depend on. When local variable entanglement blocks extraction, Replace Method with Method Object provides an alternative path.

---

*Based on: Refactoring (Fowler, 1999)*
