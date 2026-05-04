# Law of Demeter

## Statement
The Law of Demeter dictates that a method should invoke operations only on objects it receives as parameters, creates locally, or holds as direct fields -- never on objects obtained through a chain of accessor calls. Colloquially known as "Don't Talk to Strangers," the principle restricts each module to communicating with its immediate collaborators, preventing it from depending on the internal structure of objects it does not directly own. This constraint reduces coupling and localizes the impact of structural changes.

## Origin
Karl Lieberherr and Ian Holland formulated the Law of Demeter at Northeastern University and presented it at OOPSLA in 1987. The name references the Demeter project, an early effort in aspect-oriented programming that explored how to structure object-oriented systems for maximum adaptability. The original paper, "Assuring Good Style for Object-Oriented Programs," demonstrated both theoretically and empirically that adherence to the law correlated with fewer defects and easier maintenance. The principle has since become a staple of object-oriented design education, often taught alongside encapsulation and information hiding.

## Software Implications
A classic violation looks like `order.getCustomer().getAddress().getCity().toUpperCase()`. This chain forces the calling code to know that an Order holds a Customer, that a Customer holds an Address, and that an Address holds a city string. If any of those relationships change -- perhaps Address is replaced by a Location value object -- every call site that traverses this chain breaks. A Demeter-compliant alternative provides `order.getShippingCity()`, encapsulating the traversal inside the Order class where the structural knowledge belongs.

The law extends beyond getter chains. Service objects that reach through a repository to obtain a database connection to execute raw SQL similarly violate Demeter by coupling the service to the repository's internal persistence mechanism. Injecting a data-access interface directly into the service keeps the dependency explicit and the service unaware of how data is stored. In distributed systems, a client that calls a gateway that calls a downstream service that calls a third service introduces a deep traversal graph; providing a facade endpoint that aggregates the response behind one call follows Demeter at the API level.

Testing provides immediate feedback on Demeter violations. Code that reaches deeply into object graphs requires elaborate test setups: to test a method that calls `a.getB().getC().doWork()`, the test must construct mock objects A, B, and C wired together, which is a strong signal that the production code knows too much about the internal structure of its collaborators. Demeter-compliant code needs only the immediate collaborator as a test double, yielding simpler, more focused tests.

## Practical Guidance
- When you write a chain of two or more dots, consider whether the traversal should instead be encapsulated as a method on the first object in the chain.
- Use Tell, Don't Ask: instead of pulling data out of an object to make a decision, tell the object to perform the action itself.
- Apply the Facade pattern when a complex subsystem needs to be accessed from outside its module boundary; the facade becomes the single point of interaction.

## Common Misreadings
One common misreading applies the law so strictly that even benign accessor chains are forbidden, leading to bloated interface classes that delegate every possible operation. Reading `person.getName().length()` is not a meaningful Demeter violation in most contexts; the law targets chains that traverse structural boundaries between modules, not trivial string operations. Another mistake is treating the Law of Demeter as a universal prohibition on any knowledge of object structure; some frameworks and serialization libraries require navigating object graphs by design, and applying Demeter there would fight the framework rather than improve the code.

## Interactions
The Law of Demeter is a practical enforcement mechanism for the Dependency Inversion Principle from SOLID: by restricting an object to its immediate collaborators, the law naturally keeps dependencies abstract and local. DRY benefits from Demeter adherence because knowledge of an object's internal structure is centralized within that object rather than scattered across call sites. KISS and Demeter sometimes conflict when a strict reading of the law would add wrapper methods that exist solely to satisfy the rule; in those cases, applying the spirit of the law -- reducing coupling across module boundaries -- matters more than counting dots. The Principle of Least Astonishment is strengthened when objects hide their internals, because callers are not surprised by structural changes they never depended on.

---

*Based on: Lieberherr & Holland, OOPSLA (1987)*
