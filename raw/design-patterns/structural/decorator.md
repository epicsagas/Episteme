# Decorator

## Essence
Decorator is a structural design pattern that attaches additional responsibilities to an object dynamically by wrapping it in another object that shares the same interface. Each decorator forwards requests to the wrapped object and adds its own behavior before or after the delegation. Wrappers can be stacked arbitrarily, composing behaviors at runtime without altering the underlying object's code or creating an explosion of subclasses.

## Motivation
An e-commerce platform calculates the final price of a shopping cart through a pipeline of adjustments: base price, promotional discounts, loyalty points redemption, tax calculation, and shipping fees. Each adjustment is optional and can be combined in different orders depending on the customer's region and membership tier. Modeling every valid combination as a subclass (TaxedCart, DiscountedTaxedCart, LoyaltyDiscountedTaxedCart) creates a factorial explosion of class names, and moving an adjustment earlier in the chain requires an entirely new subclass. The pricing logic becomes scattered across a tangled inheritance tree that is painful to extend.

Decorator replaces this inheritance nightmare with composable wrappers. A base `PricingCalculator` returns the raw subtotal. A `DiscountDecorator` wraps any `PricingCalculator`, applies a promotional reduction, and returns the adjusted total. A `TaxDecorator` wraps the discounted calculator and adds tax, while a `ShippingDecorator` wraps the taxed calculator and adds delivery costs. Each decorator is independent, reusable, and stackable in any order the business requires. Adding a new adjustment, such as a currency-conversion surcharge, means writing one new decorator class and inserting it into the chain at the appropriate position.

## Participants
The component interface declares the operations that both the core object and every decorator must implement. A concrete component provides the base behavior. The base decorator class implements the component interface and holds a reference to a wrapped component object, delegating every method call to it. Concrete decorator classes extend the base decorator, adding behavior before or after the delegation. The client assembles the decorator chain, choosing which wrappers to apply and in what order, then calls the outermost decorator as if it were a plain component.

## Application

**Use when:**
- Responsibilities must be added to objects at runtime without affecting other instances of the same class
- Extending behavior through inheritance is impractical due to sealed classes, deep class trees, or a combinatorial number of feature permutations
- Behaviors should be composable, removable, or reorderable independently (logging, compression, encryption layers in a network stack)

**Prefer alternatives when:**
- Only one static extension is needed and the class is open for inheritance (a simple subclass is clearer)
- The number of optional behaviors is small and fixed (the decorator infrastructure may be overkill)
- Middle layers of a wrapper chain must be removed dynamically (removing a specific decorator from a stack is awkward)

## Consequences
Decorator keeps each responsibility in its own class, supporting the Single Responsibility Principle and making individual behaviors easy to test in isolation. Wrappers can be combined in arbitrary permutations, which is far more flexible than a fixed inheritance tree. However, the pattern introduces complexity: a decorated object is buried inside multiple wrapper layers, making it harder to inspect during debugging. The order of decorators matters (applying tax before versus after a discount yields different totals), and documenting the correct ordering is essential. Initializing the full chain can be verbose, especially when many decorators are involved, though builder utilities or dependency injection containers can mitigate this. Clients that rely on the concrete type of the core object will not recognize it once decorated, since only the component interface is preserved.

## Relations
Decorator and Adapter both wrap objects, but Adapter converts interfaces while Decorator extends behavior through the same interface. Proxy shares the wrapping structure with Decorator, yet Proxy controls access to the wrapped object (caching, lazy loading, access control) and manages its lifecycle independently, while Decorator lets the client control the composition and adds visible behavior. Composite aggregates multiple children behind a shared interface, whereas Decorator wraps a single object and adds responsibility; still, their recursive structure is similar, and a Composite of one child behaves much like a Decorator. Strategy changes an object's internal algorithm ("its guts"), while Decorator layers external behavior around it ("its skin"). Chain of Responsibility and Decorator both pass requests along a chain, but each handler in Chain of Responsibility decides independently whether to process or pass the request, while decorators always execute both their own logic and the delegation.

---

*Based on: Design Patterns (Gamma, Helm, Johnson, Vlissides, 1994)*
