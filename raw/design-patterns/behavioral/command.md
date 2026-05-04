# Command

## Essence
Command is a behavioral design pattern that encapsulates a request as a self-contained object, bundling the operation name, its parameters, and the receiver reference into a single unit. This encapsulation lets you queue operations, schedule them for later execution, log them for audit trails, and build undo and redo stacks. The invoker that triggers the command knows nothing about the actual work being performed, achieving clean separation between the caller and the execution logic.

## Motivation
Imagine a home-automation system where a physical remote control must operate lights, thermostats, and audio systems interchangeably. Without Command, each button on the remote would need direct knowledge of the device it controls, leading to a tangle of conditional logic whenever a button is reassigned. Worse, features like programming a macro button that dims the lights, lowers the thermostat, and starts music become impossible without coupling the remote to every device class. By wrapping each action in a command object, the remote merely stores and invokes commands through a uniform interface. A macro becomes a composite command holding a list of sub-commands.

The same need arises in transaction-processing systems. A funds transfer between bank accounts must be reversible if a later step fails. Recording each operation as a command object lets the system replay or roll back the entire sequence reliably.

## Participants
The pattern centers on a Command interface declaring an execution method, often supplemented with an undo method. Concrete command objects bind a specific receiver to an action, storing any parameters needed at invocation time. The invoker holds command references and decides when to call execute, without understanding what the command does. The receiver contains the actual business logic that the command delegates to. The client assembles these pieces, wiring receivers into commands and commands into invokers.

## Application

**Use when:**
- operations must be queued, scheduled, or executed remotely
- undo and redo support is required
- you need to log or audit operations for replay or compliance
- invoker objects should be decoupled from the objects performing the work

**Prefer alternatives when:**
- a request is always handled immediately by a single known handler with no need for queuing or reversal
- the added indirection would obscure simple, direct method calls with no compensating benefit

## Consequences
Command delivers strong adherence to the Single Responsibility Principle by isolating request initiation from execution, and the Open/Closed Principle by allowing new commands without changing invoker or receiver code. Undo and redo become straightforward when each command stores the state needed to reverse itself, often in cooperation with Memento. Composite commands assemble complex workflows from simple ones. The cost is a proliferation of small command classes that can feel verbose for trivial operations. Developers must also carefully manage state captured at command-creation time, since stale references to mutable receivers can cause subtle bugs during deferred execution.

## Relations
Command pairs naturally with Memento for implementing undo: the command performs the action while the memento captures the state needed for reversal. Composite builds macro commands by treating commands as tree nodes. Strategy and Command share structural similarities through object-based encapsulation, but Strategy focuses on swapping algorithms within a context, whereas Command focuses on capturing requests for deferred or repeated execution. Prototype can assist by cloning command objects for history stacks. Visitor can be viewed as an advanced form of Command that dispatches across a family of element types rather than targeting a single receiver.

---

*Based on: Design Patterns (Gamma, Helm, Johnson, Vlissides, 1994)*
