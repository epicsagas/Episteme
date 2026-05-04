# Hide Method

## Motivation

Hide Method restricts a method's visibility to the narrowest scope that satisfies all current usage, typically by changing public access to private or protected. When a public method is only invoked from within its own class or its subclass hierarchy, the broader visibility grants access that no caller exercises. That excess access creates risk: any external class can depend on the method, forming a coupling point that makes future internal refactoring harder because every change must account for hypothetical external callers.

The technique often follows the natural maturation of a class. Early in development, getters and setters might be exposed publicly to support quick prototyping. As richer behavior accumulates on the class, those low-level accessors become implementation details consumed internally by higher-level methods. At that point, leaving them public miscommunicates the class's contract, suggesting that direct field manipulation is part of the intended API when it no longer should be.

## Mechanics

1. Identify candidate methods through static analysis or IDE usage search. Any public method with zero external callers outside its inheritance chain qualifies.
2. Verify that no reflection-based or framework-driven invocation depends on the public modifier, as is common with dependency injection containers or serialization libraries.
3. Reduce visibility to protected if subclasses need access, or private if usage is confined to the declaring class.
4. Run the full test suite. If tests break because they called the method directly, rewrite those tests to exercise the behavior through the class's public contract instead.
5. Treat the refactoring as a continuous habit rather than a one-time sweep. Apply the principle of least privilege whenever a method's calling context narrows.

## Indications

**Signs suggesting this refactoring:**
- A public method has no callers outside its own class after a recent round of Extract Method or Move Method.
- IDE usage search returns results only within the class itself and its subclasses.
- A getter or setter exists purely to support a higher-level public method that orchestrates the same data.

**When to avoid:**
- The method is part of a published interface or abstract base class that external modules implement or consume.
- Framework conventions require public visibility, such as event handlers, lifecycle callbacks, or entry-point methods.
- Hiding the method would force extensive test rewrites without any corresponding design improvement.

## Trade-offs

The primary gain is reduced coupling. A private method can be renamed, restructured, or deleted without consulting any other class, which accelerates internal refactoring and lowers the risk of unintended breakage. It also sharpens the public contract: developers reading the class see only the operations that matter externally, which clarifies intent. The cost is minimal, mostly the effort of verifying call sites and updating any affected tests. In rare cases, hiding a method that tests relied on directly can reduce test granularity, but this often indicates that the test was coupled to implementation rather than behavior.

## Connections

Hide Method directly counters the Data Class smell, where a class exposes fields through public getters and setters without meaningful behavior. After hiding those accessors, the class is nudged toward encapsulation. The technique pairs naturally with Move Method and Extract Method, both of which can produce new internal methods that start life private. Replace Constructor with Factory Method often involves hiding the constructor itself via the same visibility reduction. On the flip side, making a method public again may become necessary when a previously internal utility gains broader reuse, at which point the decision reverses through a complementary visibility expansion.

---

*Based on: Refactoring (Fowler, 1999)*
