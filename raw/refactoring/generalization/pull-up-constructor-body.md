# Pull Up Constructor Body

## Motivation

Pull Up Constructor Body is a refactoring technique that consolidates shared initialization logic from subclass constructors into a shared superclass constructor. The hallmark of the problem is a set of sibling subclasses whose constructors all perform the same field assignments in their opening lines before diverging into subclass-specific setup. This repetition is easy to overlook because constructors are not inherited in most object-oriented languages, so developers tend to treat each constructor as a self-contained unit rather than scanning for cross-constructor patterns.

The technique differs from Pull Up Method in an important way: constructors cannot be inherited in Java, C#, or PHP, and the parent constructor runs automatically before the subclass body in many runtimes. These constraints mean that the shared logic must be physically placed in a parent constructor and invoked explicitly via `super()` or equivalent, rather than simply moved as a regular method would be.

## Mechanics

1. Inspect the constructors of each subclass and identify lines of initialization code that are identical across all of them. Focus on the earliest statements, since parent constructors execute before subclass bodies in most runtimes.
2. Create a constructor in the superclass that accepts exactly the parameters needed for the shared initialization -- no more. Avoid passing subclass-specific parameters through the parent constructor.
3. Move the shared initialization statements into the new superclass constructor.
4. In each subclass constructor, add a call to the parent constructor (`super(...)`, `parent::__construct(...)`, or equivalent) as the first statement, passing the relevant arguments. Delete the initialization lines that the parent now handles.
5. Retain only subclass-specific initialization in the child constructors.
6. Run the test suite to confirm that object state after construction is unchanged.

## Indications

**Signs suggesting this refactoring:**
- Two or more subclass constructors begin with identical field assignments before branching into specialized setup.
- A new subclass is being added and its constructor duplicates lines already present in sibling constructors.
- Initialization order bugs appear because developers forgot to replicate a setup step that other subclasses perform.

**When to avoid:**
- The shared initialization depends on values computed by the subclass constructor. Parent constructors execute first, so they cannot reference subclass-local variables. In this case, consider restructuring the initialization so the subclass computes values and passes them as arguments to the parent.
- Only one subclass exists; generalizing constructor behavior prematurely for a single child adds complexity without immediate benefit.

## Trade-offs

Centralizing shared construction logic yields the same maintenance advantage as any deduplication: a field rename or a validation rule change requires editing one constructor instead of many. It also establishes a convention that makes adding future subclasses easier, since the parent constructor documents exactly which fields the base concept requires. The cost is a slight increase in indirection: developers must navigate to the parent constructor to understand the full initialization sequence. In languages where the parent constructor runs implicitly, developers may not realize it exists unless the `super()` call is visible. Parameter mismatch is another common pitfall; if the parent constructor's signature diverges from what subclasses actually need, the refactoring can introduce awkward parameter passing that makes constructors harder to read rather than easier.

## Connections

Pull Up Constructor Body is typically applied alongside Pull Up Field, since shared initialization usually assigns shared fields. It is a prerequisite or companion to Extract Superclass -- once fields and methods are pulled up, their construction setup should follow. The technique addresses the Duplicate Code smell in the specific context of object initialization. It also relates to Pull Up Method but operates under the language-level constraints that make constructor inheritance different from method inheritance. In languages that support constructor chaining or mixins, the same goal may be achieved through those mechanisms rather than a traditional superclass constructor.

---

*Based on: Refactoring (Fowler, 1999)*
