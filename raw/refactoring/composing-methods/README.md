# Composing Methods

**Category of Refactoring Techniques**

Techniques for improving method organization and clarity by breaking complex methods into focused, maintainable units.

## Techniques (9 total)

| Technique | Purpose |
|-----------|---------|
| [Extract Method](extract-method.md) | Move code to a separate method for clarity |
| [Inline Method](inline-method.md) | Replace unnecessary method calls with implementation |
| [Extract Variable](extract-variable.md) | Break complex expressions into self-explanatory variables |
| [Inline Temp](inline-temp.md) | Replace simple temp variables with expressions |
| [Replace Temp with Query](replace-temp-with-query.md) | Convert temp variables to method calls |
| Split Temporary Variable | Use different variables for different purposes |
| Remove Assignments to Parameters | Use local variables instead of parameters |
| Replace Method with Method Object | Transform complex methods into classes |
| Substitute Algorithm | Replace algorithm with better implementation |

## Key Principle

"Methods should be cohesive and focused. Each method should have a single, clear responsibility. When methods grow too large or handle multiple concerns, break them into smaller, well-named methods."

## When to Apply

- Methods exceed 10-20 lines of code
- Methods perform multiple distinct operations
- Code is duplicated across methods
- Method names don't reflect actual behavior
- Complex expressions reduce code readability
- Temporary variables accumulate in methods
- Algorithm needs modification but is deeply nested

## Common Patterns

1. Start with Extract Method to isolate concerns
2. Use Extract Variable to clarify complex expressions
3. Apply Replace Temp with Query to eliminate intermediate variables
4. Consider Replace Method with Method Object for very complex logic
5. Use Substitute Algorithm only after code is well-organized

## Benefits

- **Readability** - Clear method names document intent
- **Reusability** - Extracted methods can be used elsewhere
- **Testability** - Smaller methods are easier to test
- **Maintainability** - Changes affect fewer lines
- **Performance** - Enables better optimization opportunities
