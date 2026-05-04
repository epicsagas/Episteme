# Refactoring Techniques - Complete Reference

## What is Refactoring?

A systematic process of improving code without creating new functionality — transforming a mess into clean code and simple design.

## Core Concepts

### Dirty Code
Accumulated technical debt resulting from inexperience, tight deadlines, and shortcuts — leading to reduced maintainability over time.

### Clean Code
Code that is easy to read, understand, and maintain — achievable through systematic, incremental refactoring.

### Code Smells
Indicators of problems addressable during refactoring:
- **Bloaters:** Long Method, Large Class, Primitive Obsession, Long Parameter List, Data Clumps
- **Object-Orientation Abusers:** Switch Statements, Temporary Field, Refused Bequest, Alternative Classes
- **Change Preventers:** Divergent Change, Shotgun Surgery, Parallel Inheritance Hierarchies
- **Dispensables:** Comments, Duplicate Code, Lazy Class, Data Class, Dead Code, Speculative Generality
- **Couplers:** Feature Envy, Inappropriate Intimacy, Message Chains, Middle Man

### Refactoring Process
Step-by-step execution with testing after each change to ensure correctness.

---

## Refactoring Techniques by Category

### 1. Composing Methods (9 Techniques)
Improving method composition and clarity:

1. **Extract Method** - Move code to a separate method for clarity
2. **Inline Method** - Replace calls with method content when method is unnecessary
3. **Extract Variable** - Break complex expressions into self-explanatory variables
4. **Inline Temp** - Replace simple temp variables with their expressions
5. **Replace Temp with Query** - Convert temp variables to method calls
6. **Split Temporary Variable** - Use different variables for different purposes
7. **Remove Assignments to Parameters** - Use local variables instead of parameters
8. **Replace Method with Method Object** - Transform complex methods into classes
9. **Substitute Algorithm** - Replace algorithm implementation with better one

### 2. Moving Features Between Objects (8 Techniques)
Organizing code across appropriate classes:

1. **Move Method** - Move method to more appropriate class
2. **Move Field** - Move field to more appropriate class
3. **Extract Class** - Create new class for related functionality
4. **Inline Class** - Merge underutilized class with another
5. **Hide Delegate** - Create delegating method to reduce dependencies
6. **Remove Middle Man** - Remove unnecessary delegating methods
7. **Introduce Foreign Method** - Add needed method to utility class
8. **Introduce Local Extension** - Create local wrapper for utility class

### 3. Organizing Data (15 Techniques)
Improving data structure and access:

1. **Self Encapsulate Field** - Create getter/setter for field access
2. **Replace Data Value with Object** - Convert data value to object
3. **Change Value to Reference** - Convert identical objects to single reference
4. **Change Reference to Value** - Convert reference object to value object
5. **Replace Array with Object** - Replace array with object having named fields
6. **Duplicate Observed Data** - Separate GUI data from domain data
7. **Change Unidirectional Association to Bidirectional** - Add missing association
8. **Change Bidirectional Association to Unidirectional** - Remove unused association
9. **Replace Magic Number with Symbolic Constant** - Use named constants
10. **Encapsulate Field** - Make field private with access methods
11. **Encapsulate Collection** - Make collection read-only and provide add/remove methods
12. **Replace Type Code with Class** - Create class for type code values
13. **Replace Type Code with Subclasses** - Create subclasses for code values
14. **Replace Type Code with State/Strategy** - Use state object for behavior
15. **Replace Subclass with Fields** - Move methods to parent class as fields

### 4. Simplifying Conditional Expressions (8 Techniques)
Making conditionals cleaner and more maintainable:

1. **Decompose Conditional** - Break complex conditionals into methods
2. **Consolidate Conditional Expression** - Merge multiple conditionals with same result
3. **Consolidate Duplicate Conditional Fragments** - Extract common code from branches
4. **Remove Control Flag** - Replace flag variable with break/continue/return
5. **Replace Nested Conditional with Guard Clauses** - Isolate special cases first
6. **Replace Conditional with Polymorphism** - Use method overriding instead of conditions
7. **Introduce Null Object** - Return null object instead of null references
8. **Introduce Assertion** - Replace assumptions with explicit assertions

### 5. Simplifying Method Calls (14 Techniques)
Improving method interfaces and invocation:

1. **Rename Method** - Rename for clarity and meaning
2. **Add Parameter** - Add parameter for needed data
3. **Remove Parameter** - Remove unused parameters
4. **Separate Query from Modifier** - Split method returning values from modifying state
5. **Parameterize Method** - Use parameter instead of multiple methods
6. **Replace Parameter with Explicit Methods** - Create separate methods instead of parameter variants
7. **Preserve Whole Object** - Pass entire object instead of extracted values
8. **Replace Parameter with Method Call** - Call method instead of passing parameter
9. **Introduce Parameter Object** - Replace parameter groups with object
10. **Remove Setting Method** - Remove setter for immutable fields
11. **Hide Method** - Make method private/protected when unused externally
12. **Replace Constructor with Factory Method** - Use factory for complex creation
13. **Replace Error Code with Exception** - Throw exception instead of error codes
14. **Replace Exception with Test** - Use conditional test instead of exception

### 6. Dealing with Generalization (12 Techniques)
Managing inheritance and abstraction hierarchies:

1. **Pull Up Field** - Move identical field to superclass
2. **Pull Up Method** - Move identical method to superclass
3. **Pull Up Constructor Body** - Move common constructor code to superclass
4. **Push Down Method** - Move method to subclass when used only there
5. **Push Down Field** - Move field to subclass when used only there
6. **Extract Subclass** - Create subclass for specialized behavior
7. **Extract Superclass** - Create superclass for common behavior
8. **Extract Interface** - Create interface for shared behavior
9. **Collapse Hierarchy** - Merge similar superclass and subclass
10. **Form Template Method** - Move algorithm structure to superclass
11. **Replace Inheritance with Delegation** - Use composition instead of inheritance
12. **Replace Delegation with Inheritance** - Use inheritance instead of composition

---

---

*Based on: Fowler, "Refactoring: Improving the Design of Existing Code" (Addison-Wesley, 1999)*

---

## Total: 66 Refactoring Techniques

All techniques are organized to address specific code smells and improve code quality across:
- **Readability** - Making code easier to understand
- **Maintainability** - Reducing cost of change
- **Reusability** - Enabling code sharing
- **Testability** - Facilitating unit testing
- **Design** - Supporting better object-oriented design

## How to Choose a Refactoring

1. Identify the code smell (e.g., Long Method, Duplicate Code)
2. Select appropriate technique from the list above
3. Apply refactoring with unit tests
4. Verify behavior hasn't changed
5. Repeat as needed

## Best Practices

- Refactor in small steps
- Run tests after each change
- Keep one change per refactoring
- Don't combine refactoring with new features
- Review changes with team
- Document refactoring motivation
