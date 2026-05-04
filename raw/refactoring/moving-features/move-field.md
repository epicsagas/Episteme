# Move Field

## Motivation

Move Field repositions a data attribute from the class where it is declared to the class where it is most heavily used. The canonical signal is a field that is read and written primarily by methods of another class, while the owning class rarely touches it. This misplacement forces the owning class to expose the field through accessors that exist solely so external classes can reach it, inflating the interface and weakening encapsulation. The field's conceptual home is elsewhere — it describes state that matters to the target class's behavior, not to the class that happens to hold it.

The refactoring frequently appears as a supporting step within Extract Class and Inline Class. When responsibilities migrate between classes, the data that supports those responsibilities must follow. A field left behind after its methods have moved becomes an orphan: still declared in the old class, accessed only through chains of getters from the new class, creating unnecessary coupling between the two.

## Mechanics

1. Encapsulate the field if it has public or package-level visibility. Replace direct access with getter and setter methods on the owning class. This step is essential because it establishes the indirection layer that allows the field's location to change without affecting callers.

2. Create the field in the target class with appropriate accessors. Match the type, visibility, and initialization semantics of the original field. Decide whether the field should be an instance variable on existing target objects or whether the target needs a new data structure to hold it.

3. Establish a reference from the original class to the target class. Use an existing field or method if one already provides access to the target instance. If no path exists, add a field or parameter that connects the two. Prefer a unidirectional link to avoid circular dependencies.

4. Redirect the original class's accessors to delegate to the target class. The getter on the original class now forwards to the target's getter; the setter forwards to the target's setter. External callers continue invoking the same methods on the same objects, unaware that the data has moved.

5. Replace direct uses of the field throughout the codebase. Each reference to the old getter or setter on the original class should be updated to call the target class directly when the caller already has access to the target. Use the compiler or a static analysis tool to locate every reference.

6. Remove the field from the original class once all references have been redirected and the delegating accessors are no longer needed. If the accessors are still called by external code, keep them as delegating facades and remove them in a later cleanup pass.

## Indications

**Signs suggesting this refactoring:**
- A field is accessed more frequently by another class than by its own class's methods
- Feature Envy: methods in one class repeatedly access a field of another class through getters, suggesting the field belongs in the envious class
- Shotgun Surgery: changes to the field's semantics require coordinated edits in a distant class because the field's declaration and its primary consumers are separated
- Parallel Inheritance Hierarchies where fields in one hierarchy mirror fields in another, and consolidating would reduce duplication

**When to avoid:**
- The field is used evenly by multiple classes, and moving it would merely shift the coupling rather than eliminate it
- Moving the field would create a circular dependency between the two classes
- The field participates in the original class's equality, hash code, or serialization contract, and relocating it would break those invariants without careful coordination

## Trade-offs

Move Field is a focused, low-risk refactoring that strengthens cohesion by colocating data with the behavior that depends on it. When a field clearly belongs in one class, the move reduces the indirection of cross-class access and tightens each class's responsibility boundary. The cost is minimal: a reference from the original class to the target and a brief period during the move where the field exists in two places.

The technique can backfire when the field's ownership is genuinely shared — when two classes need equal access and neither is the primary consumer. Moving the field to either class simply reverses the direction of coupling. In such cases, the field may deserve its own class, or the relationship between the two classes may need restructuring rather than a simple relocation. Move Field is a surgical tool; when the diagnosis is uncertain, a broader refactoring like Extract Class may address the root cause more effectively.

## Connections

Move Field is the data counterpart to Move Method, and the two are frequently applied together when relocating a responsibility wholesale. Both support Extract Class by providing the mechanical steps to transfer features into a new structural unit. Inline Class uses Move Field in reverse, pulling fields from a donor class into a host before the donor is deleted. The refactoring addresses Inappropriate Intimacy by reducing the need for one class to reach deeply into another's state. It also mitigates Shotgun Surgery when changes to the field's definition currently force edits across multiple classes.

---

*Based on: Refactoring (Fowler, 1999)*
