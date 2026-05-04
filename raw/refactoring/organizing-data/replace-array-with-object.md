# Replace Array with Object

## Motivation

An array used to store heterogeneous data, where each index carries a distinct meaning, forces developers to remember which position holds which value. This encoding is fragile: swapping two elements or misremembering an index produces subtle bugs that type checkers cannot catch, because the array treats every slot as the same type. Named fields on an object, by contrast, are self-documenting and type-safe, and they provide a natural home for the behaviors that operate on the data.

Arrays shine when every element has identical meaning, such as a list of temperatures or a sequence of pixel colors. When an array begins to accumulate different kinds of data, each accessed by a magic index constant, the design has drifted into territory where an object offers clearer intent, better tooling support, and room to grow.

## Mechanics

1. Create a new class representing the data currently stored in the array. Declare a public field for the array itself, or for each element individually, whichever is easier to initialize. The goal is to establish a container that can hold the same data under a named type.
2. Add a field of the new class type to the original class, replacing the array reference. Initialize it appropriately, copying data from the existing array.
3. For each meaningful array index, add a named getter and setter on the new class. Replace every array-index access in the codebase with a call to the corresponding accessor. This step makes the mapping from index to meaning explicit and auditable.
4. Once all external accesses have been redirected to named accessors, make the internal representation private. If the new class still stores an array internally, replace it with individual named fields, one per logical element.
5. Delete the original array and remove any index constants that have become obsolete.

## Indications

**Signs suggesting this refactoring:**
- An array is accessed using named constants or numeric literals as indices, and each position holds conceptually different data, such as a name at index 0 and an age at index 1.
- Methods that process the array must unpack it into local variables with meaningful names before doing useful work, suggesting that the array is merely an awkward transport mechanism.
- The array's length is fixed and known at compile time, and new elements are never appended dynamically.

**When to avoid:**
- The array genuinely stores a homogeneous list where every element has the same meaning and is processed uniformly, such as a buffer of bytes or a list of scores.
- The array is used for serialization or interop with a library that expects array-shaped data, and wrapping it in an object would create an impedance mismatch.

## Trade-offs

An object consumes slightly more memory and construction time than a bare array, but this cost is negligible for the small, fixed-size arrays this refactoring targets. The real trade-off is in API surface: an object with named fields is more verbose to construct than an array literal, though builder patterns or keyword arguments can mitigate this. On the benefit side, named fields enable compiler-enforced type checking, IDE autocompletion, and straightforward documentation. They also provide a natural attachment point for methods that operate on the grouped data, eliminating the need for external utility functions that accept loosely coupled parameters.

## Connections

This refactoring is a specialized form of Replace Data Value with Object, applied when the primitive data structure happens to be an array. It addresses the Primitive Obsession smell by replacing a generic container with a domain-specific type. After the object is established, Introduce Parameter Object can use it to bundle related arguments into a single parameter. Move Method may follow, relocating behavior that currently lives outside the new class into the class itself. The resulting object may eventually warrant Change Value to Reference if shared identity becomes necessary.

---

*Based on: Refactoring (Fowler, 1999)*
