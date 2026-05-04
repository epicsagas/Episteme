# Introduce Assertion

## Motivation

Introduce Assertion converts implicit assumptions buried in code comments or developer knowledge into explicit runtime checks that halt execution when violated. Most methods operate under preconditions: a parameter must be non-null, an object field must fall within a valid range, or a collection must not be empty. When these assumptions are left unstated, the code may fail silently, produce corrupt data, or crash far from the actual source of the problem. Assertions surface these invariants at the point where they matter, turning latent defects into immediate, diagnosable failures. The technique is especially valuable in methods that receive input from external callers, in shared library code where the consumer may not understand every constraint, and in complex algorithms where intermediate state must satisfy specific properties to guarantee correctness.

## Mechanics

1. Survey the method for implicit assumptions, paying particular attention to comments that describe expected values, parameters that are used without null checks, and fields that are assumed to be initialized. Each of these signals a candidate assertion.
2. For each identified assumption, write an assertion statement that evaluates the expected condition and raises an error if it is false. Use the language's native assertion facility when available, or a lightweight assertion library.
3. Ensure the assertion is purely diagnostic: it must not alter program state, produce side effects, or change the control flow under normal operation. If the assertion itself calls a method, verify that method has no side effects.
4. Position assertions as early as possible in the method, ideally at the top, so they fail before any work is done on invalid input. For mid-method invariants, place the assertion immediately before the code that depends on the condition.
5. Limit assertions to conditions that must hold for correctness. Do not assert conditions that might plausibly fail due to user input or transient system state; use proper exception handling for those cases instead.
6. Verify that assertions can be disabled in production builds without changing behavior, if the language or framework supports that capability.

## Indications

**Signs suggesting this refactoring:**
- Comments that describe what a value should be, such as "must be positive" or "cannot be null at this point"
- Parameters that are dereferenced without any defensive check, implying the caller is trusted to provide valid input
- Methods that produce nonsensical results for certain input ranges but contain no validation at the entry point
- Complex calculations that assume intermediate results satisfy specific properties, such as a sorted list or a non-empty set

**When to avoid:**
- The condition can be violated by external user input, network failures, or other recoverable events where an exception with retry logic is more appropriate
- The assertion duplicates existing validation that already throws a meaningful exception
- The check is expensive enough to affect performance in hot paths, and no cheaper invariant is available
- The team convention explicitly reserves assertions for testing and discourages runtime use

## Trade-offs

Assertions provide a safety net that catches programming errors at their origin rather than downstream, where symptoms become confusing and diagnosis expensive. They also serve as executable documentation: a future reader can see exactly what the method requires without parsing comments that may be outdated. The cost is additional code that must be maintained and that can clutter method signatures if overused. Assertions that fire in production can crash the application, which is desirable for logic errors but catastrophic if the assertion guards against a condition that is merely unlikely rather than impossible. Teams must also decide whether to ship assertions in production builds, weighing the early-failure benefit against the risk of assertion failures caused by edge cases not encountered during testing. Use assertions sparingly for genuine invariants and rely on exceptions for conditions that external actors can trigger.

## Connections

Introduce Assertion addresses the Comments smell by replacing prose descriptions of preconditions with machine-verified checks. It complements Decompose Conditional by asserting the invariants that extracted condition methods rely on. Replace Error Code with Exception operates in a related space but handles expected failure modes rather than programmer errors. Introduce Null Object can reduce the need for null-check assertions by guaranteeing that a valid object is always present. The technique also supports Design by Contract methodology, where preconditions, postconditions, and invariants define the formal interface between software components.

---

*Based on: Refactoring (Fowler, 1999)*
