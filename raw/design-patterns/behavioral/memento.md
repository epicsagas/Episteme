# Memento

## Essence
Memento is a behavioral design pattern that captures and externalizes an object's internal state so that the object can be restored to that state later, all without violating encapsulation. The object whose state is being saved, called the originator, creates a memento containing a snapshot of its fields. A separate caretaker object stores the memento and returns it when a rollback is needed, but the caretaker never inspects or modifies the memento's contents.

## Motivation
Think of a collaborative document-editing platform that tracks changes from multiple users in real time. Each keystroke modifies shared state, and users expect to undo their own edits without losing changes made by collaborators. Storing the full document state after every edit is one approach, but naively copying all internal fields into a history list would expose private data structures to the editing engine and create tight coupling between the document's representation and the undo system. If the document's internal model changes, every snapshot format must change too.

Memento resolves this by letting the document itself produce opaque snapshot objects that only it can interpret. The undo manager holds these snapshots in a stack but cannot read them, preserving both encapsulation and the ability to revert to any prior point. Collaborative scenarios benefit further because each user's undo stack holds independent snapshots without interfering with others.

## Participants
The Originator is the object whose state is snapshot and restored. It creates mementos capturing its current internal state and uses previously created mementos to revert. The Memento is a value object that stores the originator's state immutably; ideally only the originator can access its contents. The Caretaker maintains a history of mementos, requesting snapshots from the originator at appropriate moments and triggering restoration when needed, without ever examining memento internals.

## Application

**Use when:**
- you must implement undo, rollback, or checkpoint-restore functionality
- direct serialization of an object's state would expose private fields that should remain encapsulated
- a system needs to recover from failed operations by restoring to a known-good state

**Prefer alternatives when:**
- the object's state is trivial and a simple field copy suffices without encapsulation concerns
- memory pressure is high and storing full snapshots for every state change is prohibitive, in which case delta-based or Command-based undo may be more appropriate

## Consequences
Memento preserves encapsulation by keeping state details within the originator's control, which is its primary advantage over naïve field-copying approaches. It simplifies the originator because history management is delegated to the caretaker. The drawbacks are significant memory consumption when snapshots are large or frequent, since each memento holds a complete copy of the originator's state. In dynamic languages, enforcing memento immutability is difficult because reflection can bypass access restrictions. Caretakers must also manage memento lifecycles carefully to avoid holding stale references that prevent garbage collection.

## Relations
Memento is most often paired with Command, where each command stores a memento before executing so that undo restores the prior state. Iterator can use mementos to save traversal position for checkpoint-and-resume iteration. Prototype offers an alternative snapshot mechanism through cloning, which is simpler but does not provide the same encapsulation guarantees. State may store mementos when transitioning between state objects requires the ability to roll back.

---

*Based on: Design Patterns (Gamma, Helm, Johnson, Vlissides, 1994)*
