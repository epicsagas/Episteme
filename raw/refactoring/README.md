# Refactoring Techniques - Complete Reference

A comprehensive collection of 66 refactoring techniques organized into six categories. Each technique addresses specific code smells and improves code quality through systematic transformation.

## Quick Start

**See:** [Refactoring Overview](refactoring-overview.md) for complete list of all 66 techniques with categorization and descriptions.

## Categories

### 1. Composing Methods (9 Techniques)
Improving method organization and clarity by breaking complex methods into focused units.

| Technique | Description |
|-----------|-------------|
| [Extract Method](composing-methods/extract-method.md) | Move code to a separate method for clarity |
| [Inline Method](composing-methods/inline-method.md) | Replace unnecessary method calls with implementation |
| [Extract Variable](composing-methods/extract-variable.md) | Break complex expressions into self-explanatory variables |
| [Inline Temp](composing-methods/inline-temp.md) | Replace simple temp variables with expressions |
| [Replace Temp with Query](composing-methods/replace-temp-with-query.md) | Convert temp variables to method calls |
| [Split Temporary Variable](composing-methods/split-temporary-variable.md) | Use different variables for different purposes |
| [Remove Assignments to Parameters](composing-methods/remove-assignments-to-parameters.md) | Use local variables instead of parameters |
| [Replace Method with Method Object](composing-methods/replace-method-with-method-object.md) | Transform complex methods into classes |
| [Substitute Algorithm](composing-methods/substitute-algorithm.md) | Replace algorithm with better implementation |

### 2. Moving Features Between Objects (8 Techniques)
Organizing code across the most appropriate classes through field and method relocation.

| Technique | Description |
|-----------|-------------|
| [Move Method](moving-features/move-method.md) | Relocate methods to appropriate classes |
| [Move Field](moving-features/move-field.md) | Relocate fields to appropriate classes |
| [Extract Class](moving-features/extract-class.md) | Create new class for related functionality |
| [Inline Class](moving-features/inline-class.md) | Merge class into another when too small |
| [Hide Delegate](moving-features/hide-delegate.md) | Encapsulate delegation relationships |
| [Remove Middle Man](moving-features/remove-middle-man.md) | Expose delegated objects directly |
| [Introduce Foreign Method](moving-features/introduce-foreign-method.md) | Add method to external class interface |
| [Introduce Local Extension](moving-features/introduce-local-extension.md) | Extend class with wrapper or subclass |

### 3. Organizing Data (15 Techniques)
Improving data structure representation and access patterns.

| Technique | Description |
|-----------|-------------|
| [Self Encapsulate Field](organizing-data/self-encapsulate-field.md) | Access fields through getters/setters |
| [Replace Data Value with Object](organizing-data/replace-data-value-with-object.md) | Convert primitive to object |
| [Change Value to Reference](organizing-data/change-value-to-reference.md) | Share object instances as references |
| [Change Reference to Value](organizing-data/change-reference-to-value.md) | Convert reference to immutable value |
| [Replace Array with Object](organizing-data/replace-array-with-object.md) | Use object instead of array |
| [Duplicate Observed Data](organizing-data/duplicate-observed-data.md) | Separate domain from presentation |
| [Change Unidirectional Association to Bidirectional](organizing-data/change-unidirectional-association-to-bidirectional.md) | Add back-pointer |
| [Change Bidirectional Association to Unidirectional](organizing-data/change-bidirectional-association-to-unidirectional.md) | Remove back-pointer |
| [Replace Magic Number with Symbolic Constant](organizing-data/replace-magic-number-with-symbolic-constant.md) | Use named constants |
| [Encapsulate Field](organizing-data/encapsulate-field.md) | Make field private with accessors |
| [Encapsulate Collection](organizing-data/encapsulate-collection.md) | Return copy of collection |
| [Replace Type Code with Class](organizing-data/replace-type-code-with-class.md) | Convert type code to class |
| [Replace Type Code with Subclasses](organizing-data/replace-type-code-with-subclasses.md) | Use inheritance for types |
| [Replace Type Code with State/Strategy](organizing-data/replace-type-code-with-state-strategy.md) | Use state/strategy pattern |
| [Replace Subclass with Fields](organizing-data/replace-subclass-with-fields.md) | Replace subclasses with fields |

### 4. Simplifying Conditional Expressions (8 Techniques)
Making conditional logic cleaner, easier to understand, and more maintainable.

| Technique | Description |
|-----------|-------------|
| [Decompose Conditional](simplifying-conditionals/decompose-conditional.md) | Extract complex conditional logic |
| [Consolidate Conditional Expression](simplifying-conditionals/consolidate-conditional-expression.md) | Combine similar conditions |
| [Consolidate Duplicate Conditional Fragments](simplifying-conditionals/consolidate-duplicate-conditional-fragments.md) | Move duplicate code outside conditional |
| [Remove Control Flag](simplifying-conditionals/remove-control-flag.md) | Use break/return instead of flags |
| [Replace Nested Conditional with Guard Clauses](simplifying-conditionals/replace-nested-conditional-with-guard-clauses.md) | Use early returns |
| [Replace Conditional with Polymorphism](simplifying-conditionals/replace-conditional-with-polymorphism.md) | Use polymorphism instead of switch |
| [Introduce Null Object](simplifying-conditionals/introduce-null-object.md) | Replace null checks with null object |
| [Introduce Assertion](simplifying-conditionals/introduce-assertion.md) | Document assumptions with assertions |

### 5. Simplifying Method Calls (14 Techniques)
Improving method interfaces, reducing parameters, and clarifying method invocation.

| Technique | Description |
|-----------|-------------|
| [Rename Method](simplifying-calls/rename-method.md) | Give method more meaningful name |
| [Add Parameter](simplifying-calls/add-parameter.md) | Add parameter for needed information |
| [Remove Parameter](simplifying-calls/remove-parameter.md) | Remove unused parameter |
| [Separate Query from Modifier](simplifying-calls/separate-query-from-modifier.md) | Split methods that return and modify |
| [Parameterize Method](simplifying-calls/parameterize-method.md) | Use parameter instead of similar methods |
| [Replace Parameter with Explicit Methods](simplifying-calls/replace-parameter-with-explicit-methods.md) | Create separate methods for each case |
| [Preserve Whole Object](simplifying-calls/preserve-whole-object.md) | Pass entire object instead of values |
| [Replace Parameter with Method Call](simplifying-calls/replace-parameter-with-method-call.md) | Call method instead of passing value |
| [Introduce Parameter Object](simplifying-calls/introduce-parameter-object.md) | Group parameters into object |
| [Remove Setting Method](simplifying-calls/remove-setting-method.md) | Make field immutable |
| [Hide Method](simplifying-calls/hide-method.md) | Make method private |
| [Replace Constructor with Factory Method](simplifying-calls/replace-constructor-with-factory-method.md) | Use factory method for creation |
| [Replace Error Code with Exception](simplifying-calls/replace-error-code-with-exception.md) | Throw exception instead of error code |
| [Replace Exception with Test](simplifying-calls/replace-exception-with-test.md) | Use conditional instead of exception |

### 6. Dealing with Generalization (12 Techniques)
Managing inheritance hierarchies and abstraction levels for better design.

| Technique | Description |
|-----------|-------------|
| [Pull Up Field](generalization/pull-up-field.md) | Move field to superclass |
| [Pull Up Method](generalization/pull-up-method.md) | Move method to superclass |
| [Pull Up Constructor Body](generalization/pull-up-constructor-body.md) | Move constructor logic to superclass |
| [Push Down Method](generalization/push-down-method.md) | Move method to subclass |
| [Push Down Field](generalization/push-down-field.md) | Move field to subclass |
| [Extract Subclass](generalization/extract-subclass.md) | Create subclass for specific behavior |
| [Extract Superclass](generalization/extract-superclass.md) | Create superclass for common behavior |
| [Extract Interface](generalization/extract-interface.md) | Extract interface from class |
| [Collapse Hierarchy](generalization/collapse-hierarchy.md) | Merge superclass and subclass |
| [Form Template Method](generalization/form-template-method.md) | Create template method pattern |
| [Replace Inheritance with Delegation](generalization/replace-inheritance-with-delegation.md) | Use composition over inheritance |
| [Replace Delegation with Inheritance](generalization/replace-delegation-with-inheritance.md) | Use inheritance when appropriate |

## Code Smells Addressed

Refactoring techniques target these code quality indicators:

### Bloaters
- Long Method
- Large Class
- Primitive Obsession
- Long Parameter List
- Data Clumps

### Object-Orientation Abusers
- Switch Statements
- Temporary Field
- Refused Bequest
- Alternative Classes with Different Interfaces

### Change Preventers
- Divergent Change
- Shotgun Surgery
- Parallel Inheritance Hierarchies

### Dispensables
- Comments
- Duplicate Code
- Lazy Class
- Data Class
- Dead Code
- Speculative Generality

### Couplers
- Feature Envy
- Inappropriate Intimacy
- Message Chains
- Middle Man

## Refactoring Process

1. **Identify** - Spot code smells in the codebase
2. **Select** - Choose appropriate refactoring technique
3. **Test** - Establish baseline tests before refactoring
4. **Execute** - Apply refactoring systematically
5. **Verify** - Run tests to confirm behavior unchanged
6. **Review** - Get peer feedback on changes
7. **Commit** - Document motivation in commit message

## Key Principles

- **Improve Without Changing Behavior** - Refactoring never adds new functionality
- **Small Steps** - Apply one refactoring at a time
- **Test Coverage** - Always have tests before refactoring
- **Team Communication** - Discuss refactoring goals with team
- **Continuous Improvement** - Regular refactoring prevents code decay

## Directory Structure

```
refactoring/
├── README.md (this file)
├── refactoring-overview.md (complete technique reference)
├── composing-methods/          # 9 techniques
│   ├── extract-method.md
│   ├── inline-method.md
│   ├── extract-variable.md
│   ├── inline-temp.md
│   ├── replace-temp-with-query.md
│   ├── split-temporary-variable.md
│   ├── remove-assignments-to-parameters.md
│   ├── replace-method-with-method-object.md
│   └── substitute-algorithm.md
├── moving-features/            # 8 techniques
│   ├── move-method.md
│   ├── move-field.md
│   ├── extract-class.md
│   ├── inline-class.md
│   ├── hide-delegate.md
│   ├── remove-middle-man.md
│   ├── introduce-foreign-method.md
│   └── introduce-local-extension.md
├── organizing-data/            # 15 techniques
│   ├── self-encapsulate-field.md
│   ├── replace-data-value-with-object.md
│   ├── change-value-to-reference.md
│   ├── change-reference-to-value.md
│   ├── replace-array-with-object.md
│   ├── duplicate-observed-data.md
│   ├── change-unidirectional-association-to-bidirectional.md
│   ├── change-bidirectional-association-to-unidirectional.md
│   ├── replace-magic-number-with-symbolic-constant.md
│   ├── encapsulate-field.md
│   ├── encapsulate-collection.md
│   ├── replace-type-code-with-class.md
│   ├── replace-type-code-with-subclasses.md
│   ├── replace-type-code-with-state-strategy.md
│   └── replace-subclass-with-fields.md
├── simplifying-conditionals/   # 8 techniques
│   ├── decompose-conditional.md
│   ├── consolidate-conditional-expression.md
│   ├── consolidate-duplicate-conditional-fragments.md
│   ├── remove-control-flag.md
│   ├── replace-nested-conditional-with-guard-clauses.md
│   ├── replace-conditional-with-polymorphism.md
│   ├── introduce-null-object.md
│   └── introduce-assertion.md
├── simplifying-calls/          # 14 techniques
│   ├── rename-method.md
│   ├── add-parameter.md
│   ├── remove-parameter.md
│   ├── separate-query-from-modifier.md
│   ├── parameterize-method.md
│   ├── replace-parameter-with-explicit-methods.md
│   ├── preserve-whole-object.md
│   ├── replace-parameter-with-method-call.md
│   ├── introduce-parameter-object.md
│   ├── remove-setting-method.md
│   ├── hide-method.md
│   ├── replace-constructor-with-factory-method.md
│   ├── replace-error-code-with-exception.md
│   └── replace-exception-with-test.md
└── generalization/             # 12 techniques
    ├── pull-up-field.md
    ├── pull-up-method.md
    ├── pull-up-constructor-body.md
    ├── push-down-method.md
    ├── push-down-field.md
    ├── extract-subclass.md
    ├── extract-superclass.md
    ├── extract-interface.md
    ├── collapse-hierarchy.md
    ├── form-template-method.md
    ├── replace-inheritance-with-delegation.md
    └── replace-delegation-with-inheritance.md
```

**Total: 66 refactoring techniques across 6 categories**

## About the Source

Based on: Refactoring (Fowler, 1999) and community knowledge.

## Related Resources

- **Design Patterns** - See `../design-patterns/` for 22 Gang of Four design patterns
- **Software Engineering Laws** - See `../software-engineering/` for 56 principles and laws
- **Syntagma Knowledge Graph** - All techniques are indexed with semantic embeddings and relational metadata
