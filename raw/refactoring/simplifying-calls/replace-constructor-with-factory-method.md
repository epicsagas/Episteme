# Replace Constructor with Factory Method

## Motivation

Replace Constructor with Factory Method substitutes a direct constructor call with a static or class method that returns an instance, giving control over what gets instantiated and how. Constructors are limited by language rules: they must return an instance of the enclosing class, they cannot be named to describe the creation scenario, and they always produce a new object. Factory methods escape every one of these constraints, making them essential when creation logic needs flexibility beyond simple field assignment.

The most common driver is subclass selection. After applying Replace Type Code with Subclasses, code that once constructed a base class now needs to instantiate the correct subclass based on a type code or configuration value. A constructor bound to the base class cannot do this, but a factory method can examine its arguments and return whichever subclass fits. Additional motivations include returning cached instances instead of always allocating new objects, performing expensive initialization that belongs outside the constructor, and providing self-documenting creation entry points such as `User.fromToken()` or `Order.pending()`.

## Mechanics

1. Create a static factory method on the class that delegates to the existing constructor and returns the new instance. Give it a descriptive name that communicates the creation scenario.
2. Replace every direct constructor call with an invocation of the factory method. Search the entire codebase to ensure no call site is missed.
3. Restrict the constructor's visibility to private or protected so that external code cannot bypass the factory.
4. Examine the constructor body for logic that extends beyond field assignment. Move any non-trivial initialization, validation, or resource acquisition into the factory method, keeping the constructor focused on field binding.

## Indications

**Signs suggesting this refactoring:**
- A constructor contains conditional logic that selects between different initialization paths based on its arguments.
- The class hierarchy has been reorganized via Replace Type Code with Subclasses, and callers need a single entry point that returns the correct subtype.
- Multiple constructors with similar parameter lists exist, differing only in what they initialize, and their purpose is unclear without consulting documentation.
- Object creation is expensive or should be pooled, and direct construction prevents caching or reuse.

**When to avoid:**
- The constructor does nothing beyond assigning parameters to fields, and no polymorphic creation or caching is anticipated.
- The class is a simple data holder where constructor simplicity is a feature, not a limitation.
- The team or framework conventions favor direct construction for consistency and debuggability.

## Trade-offs

Factory methods unlock polymorphic return types, named creation entry points, and object-reuse strategies that constructors cannot provide. They make creation code more readable because the method name explains the intent, and they centralize construction logic so that changes to instantiation rules affect only one place. The cost is indirection: readers must navigate to the factory method to understand what gets created. Debugging can also become slightly harder because stack traces include the factory method call instead of a direct constructor. Overuse of factory methods for classes that have straightforward constructors adds ceremony without benefit. The technique also introduces a static dependency on the concrete factory class, which can complicate testing unless the factory is itself extracted behind an interface.

## Connections

Replace Constructor with Factory Method directly enables Replace Type Code with Subclasses by providing the creation hook that selects the right subclass. It supports Change Value to Reference by allowing the factory to return an existing instance from a registry instead of allocating a new one. The technique implements the Factory Method design pattern at the class level. It pairs well with Rename Method because the factory's name serves as documentation of the creation scenario. Encapsulate Field and Remove Setting Method often precede it, locking down mutability so that the factory method becomes the sole authority over how instances are configured. On the smell side, it addresses aspects of Large Class when a constructor's complexity signals that creation logic deserves its own home.

---

*Based on: Refactoring (Fowler, 1999)*
