# State

## Essence
State is a behavioral design pattern that allows an object to change its behavior when its internal state changes, making the object appear to switch its class at runtime. Rather than sprawling conditional logic that checks a state variable before every method call, the pattern delegates state-specific behavior to separate state objects. The context holds a reference to the current state object and forwards requests to it, so transitions replace one state object with another.

## Motivation
Consider a network connection object that must behave differently depending on whether it is disconnected, connecting, authenticated, or timed out. In the disconnected state, sending data should queue it for later delivery. While connecting, sends should wait or fail gracefully. Once authenticated, sends proceed immediately. A timeout state rejects sends and forces reconnection. Encoding these rules with nested if-else chains spread across every method produces brittle code: adding a new state like "reconnecting" means hunting down every conditional branch and inserting a case.

State eliminates this by giving each network phase its own class implementing the full connection interface. The connection object simply delegates to whichever state object is active. Transitions happen by swapping the current state reference, and each state class encapsulates only the logic relevant to its phase. Adding a new state means writing one new class rather than modifying every method in the connection.

## Participants
The Context maintains a reference to the current state object and exposes the interface that clients call. It forwards all state-dependent requests to the state object. The State interface declares the methods that all concrete states must implement, matching the context's public interface. Concrete State classes provide state-specific implementations and may hold a back-reference to the context so they can trigger state transitions. Unlike Strategy, concrete states are often aware of each other and actively participate in deciding which state comes next.

## Application

**Use when:**
- an object's behavior branches heavily on a state variable and those branches are spread across many methods
- the number of states is non-trivial and new states are expected over the lifetime of the system
- state transition logic is complex enough to warrant isolating it from the context's core responsibilities

**Prefer alternatives when:**
- the state machine is trivial with only two or three states and a handful of transitions
- a simple enum with a switch statement provides sufficient clarity without the overhead of separate state classes

## Consequences
State follows the Single Responsibility Principle by localizing state-specific behavior in dedicated classes and the Open/Closed Principle by allowing new states without modifying the context or existing states. Large conditional blocks disappear from the context, improving readability. The trade-off is an increase in the number of classes: each state requires its own file or module. Transition logic scattered across state objects can be harder to survey than a single transition table, so teams often supplement the pattern with a visual state-machine diagram. For simple state machines the added class count is rarely justified.

## Relations
State is structurally identical to Strategy, since both delegate behavior to a held object through composition. The behavioral difference is that State objects can initiate transitions to other states and are aware of the broader state machine, while Strategy objects are typically independent and unaware of each other. State may combine with Memento to snapshot and restore the current state for rollback scenarios. Singleton is sometimes applied to state objects when states carry no instance-specific data and can be shared across contexts.

---

*Based on: Design Patterns (Gamma, Helm, Johnson, Vlissides, 1994)*
