# Move Method

## Motivation

Move Method relocates a method from its declaring class to the class where it is most consumed — where its data dependencies lie, where its results are used, or where its logic is most naturally understood. The telltale symptom is Feature Envy: a method that spends more time accessing another class's data and calling another class's methods than it spends working with its own class's state. The method's conceptual home has shifted, but its physical location has not followed.

Strong cohesion within a class means that its methods operate primarily on its own fields. When a method's primary collaborators are elsewhere, keeping it in its current class forces callers to orchestrate between two objects: obtaining the right instance, invoking the misplaced method, then handing the result to the object that actually needs it. Moving the method to its natural host collapses this orchestration into a single call, simplifying every caller.

## Mechanics

1. Survey the method's dependencies. Catalog every field, method, and type it references within its current class. Determine which of these will remain accessible after the move — through parameters, through the target object, or through a back-reference to the original class. Methods that are called exclusively by the method being moved are candidates for co-relocation.

2. Declare the method in the target class. Copy the signature and body, adjusting references as needed. Fields that were accessed implicitly on `this` in the original must now be accessed through the target's instance or through a parameter. If the method needs state from the original class, pass the original class as a parameter or store a reference to it in the target.

3. Create a forwarding method in the original class that delegates to the new method on the target. The forwarding method preserves the existing call sites while the move is validated. Alternatively, replace call sites directly if the target instance is readily available at each call point.

4. Replace each call to the original method with a call to the target method. If a forwarding method was introduced, update callers one at a time. After all callers target the new location directly, remove the forwarding stub.

5. Remove the original method once every call site has been redirected and no other code references it. Run the full test suite, paying attention to subclasses that may have overridden the method and to reflection-based invocations.

6. Rename the method in its new home if the original name reflected the old class's perspective rather than the target's. A method called `calculateDiscount` on a `Report` class may deserve a different name on an `Order` class.

## Indications

**Signs suggesting this refactoring:**
- Feature Envy: the method accesses more data from another class than from its own
- A method's parameters consistently include an object of another type that the method operates on heavily — this parameter is the likely true host
- Switch Statements that branch on a type code belonging to another class, suggesting the behavior belongs on that class
- Inappropriate Intimacy where two classes are entangled because behavior that belongs on one side lives on the other

**When to avoid:**
- The method references many fields of its current class, and moving it would require passing the original class as a parameter, merely shifting the coupling into a different shape
- The method is overridden in subclasses, and the target class occupies a different position in the type hierarchy, making polymorphic dispatch impossible
- Move Method would create a circular dependency between the two classes that cannot be resolved through interfaces or dependency inversion

## Trade-offs

Move Method is one of the highest-impact refactorings in terms of cohesion improvement, but it carries proportionally higher risk than smaller transformations. The method's dependency graph determines the difficulty: a method that touches three fields of its own class and one field of the target is easy to move; a method that weaves between both classes, calling methods on each, becomes a surgical challenge where every reference must be rerouted.

When the move is clean — when the method's data dependencies align with the target — the result is a simpler call graph and stronger class boundaries. When the dependencies are tangled, the moved method may need a back-reference to its original class, creating a bidirectional link that increases coupling rather than decreasing it. In such cases, the right preparation step is often Extract Method first: split the method into a piece that genuinely belongs on the target and a piece that stays, then move only the cleanly separable portion.

## Connections

Move Method is the behavioral counterpart to Move Field, and the two are applied in tandem when transferring a complete responsibility. Together they form the mechanical core of Extract Class and Inline Class — both higher-level refactorings that use move operations to relocate features between structural units. The refactoring directly addresses Feature Envy and reduces Inappropriate Intimacy by relocating behavior to the class whose data it consumes. Switch Statements often signal that a method should live on the class whose type code is being switched on. Replace Conditional with Polymorphism frequently begins with Move Method as its first step, dispatching behavior to the appropriate subclass.

---

*Based on: Refactoring (Fowler, 1999)*
