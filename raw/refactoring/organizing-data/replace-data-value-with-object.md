# Replace Data Value with Object

## Motivation

A simple primitive field, such as a string representing a phone number or a decimal standing in for a currency amount, starts its life as an innocuous data point. Over time the field accumulates related behavior: formatting rules, validation logic, partial comparisons, and derived attributes. These behaviors scatter across every class that holds the field, leading to duplicated validation code and inconsistent formatting. The root issue is that a primitive type cannot carry the behaviors that naturally belong to the concept it represents.

Extracting the field into its own class consolidates the data and its associated operations in one place. The original class holds a reference to the new object instead of the raw value, and any class that needs the same combination of data and behavior can share the new type, eliminating duplication and establishing a single authoritative implementation of the concept.

## Mechanics

1. If the original class accesses the field directly without going through accessors, first apply Self Encapsulate Field to introduce a getter and setter. This ensures that all access is funneled through methods that can later be redirected to the new object.
2. Create a new class with a constructor that accepts the primitive value and a getter that returns it. Omit a setter at this stage to enforce immutability: new values are expressed by creating new instances rather than mutating existing ones.
3. Change the field type in the original class from the primitive to the new class. Update the getter to delegate to the new object's getter, and update the setter and constructor to create a new instance of the new class wrapping the incoming primitive value.
4. Examine each call site that previously held the raw value and redirect it to work with the new object. Move any related validation, formatting, or computation methods into the new class where they naturally belong.

## Indications

**Signs suggesting this refactoring:**
- The same formatting, validation, or computation logic for a field is duplicated across multiple classes, all operating on the same primitive type.
- A field has grown ancillary data, such as a currency code alongside an amount or a unit alongside a measurement, and these companion values are stored in separate fields rather than grouped together.
- The domain vocabulary contains a noun, such as "address" or "temperature," that is currently represented only by a bare string or number.

**When to avoid:**
- The field is truly primitive with no associated behavior, no validation requirements, and no risk of duplication. Wrapping it in a class adds indirection without payoff.
- The field is used in tight loops where object allocation overhead would degrade performance, and the value is never the target of behavior.

## Trade-offs

Extracting a primitive into a dedicated object improves cohesion and eliminates duplication, but it introduces a new class and a new level of indirection. For a single consumer, the overhead of a separate class may not justify itself. The benefit scales with the number of consumers and the amount of shared behavior: once two or more classes need the same validation or formatting, the new type pays for itself. Immutability of the extracted object simplifies reasoning but means that every update requires constructing a new instance. In domains with frequent updates, this allocation pressure can be noticeable. If the extracted concept later requires shared identity across the system, follow up with Change Value to Reference.

## Connections

This refactoring is the general case from which Replace Array with Object descends as a special case. It is closely related to Extract Class, differing in that the trigger is a primitive field that has outgrown its type rather than a set of fields that should move together. Self Encapsulate Field is typically applied as a preparatory step. The resulting object may later evolve through Change Value to Reference if shared mutable identity becomes necessary. Introduce Parameter Object is a parallel technique that groups related parameters rather than replacing a single field. This refactoring addresses the Duplicate Code and Primitive Obsession smells.

---

*Based on: Refactoring (Fowler, 1999)*
