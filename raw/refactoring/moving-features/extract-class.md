# Extract Class

## Motivation

Extract Class addresses the accumulation of unrelated responsibilities within a single class, a structural degradation that compounds as systems evolve. A class that once had a clear identity gradually absorbs peripheral logic — validation rules, formatting concerns, secondary data structures — until its cohesion collapses. Fields and methods begin forming natural clusters: subsets of state that only certain methods touch, while other methods ignore them entirely. These clusters signal that one conceptual unit has fragmented into two or more distinct abstractions living under the same roof.

The degradation manifests concretely as divergent change: modifying persistence logic forces edits in the same class as changes to business rules, even though the two concerns have no structural relationship. Temporary fields that exist only to serve a narrow computation further reveal this split. When a developer must understand two unrelated domains to modify one, the class has outgrown its mandate.

## Mechanics

1. Identify a cohesive subset of fields and the methods that operate on them. Look for groups where certain fields are never referenced outside their associated method cluster, as these form the extraction candidate.

2. Create a new class with a name that captures the extracted responsibility. Naming at this stage is provisional — the right name often emerges once the extraction stabilizes.

3. Use Move Field to relocate each identified field to the new class. Apply Encapsulate Field on the original class first if direct public access exists, establishing getter and setter routes that the new class can implement.

4. Use Move Method to transfer each associated method. Start with private methods since they have the narrowest call surface, reducing the chance of breaking references. Test after each relocation rather than batching moves.

5. Establish the relationship direction between the two classes. Prefer having the original class hold a reference to the new class (unidirectional) because bidirectional links complicate garbage collection and create update synchronization obligations. Add a bidirectional link only when the new class genuinely needs to call back into the original.

6. Review and rename both classes to reflect their refined responsibilities. Reconsider the new class's visibility: if external clients benefit from direct access, make it public; if the original class should mediate all interaction, keep it private.

7. Run the full test suite after completing the extraction. Pay particular attention to equality comparisons and serialization logic, which often implicitly depend on fields that have migrated.

## Indications

**Signs suggesting this refactoring:**
- A class contains fields and methods that form two or more distinct thematic groups with no overlap
- Divergent Change appears: different categories of change require modifying the same class
- Temporary fields exist that serve only one or two methods
- Data Clumps appear where the same group of parameters travels together across multiple methods

**When to avoid:**
- The class, while large, has genuinely unified responsibilities where all fields interact with all methods
- The extraction would produce a class too thin to justify its existence, replacing one problem with a Lazy Class
- Performance-critical code where the indirection of a separate object introduces measurable overhead that cannot be mitigated

## Trade-offs

Extract Class trades class count for cohesion. The new class adds a structural unit that developers must discover, understand, and navigate. When the extracted responsibility is substantial and self-contained, this trade pays for itself immediately: the original class becomes easier to reason about, and the new class carries a clear, testable contract. When the boundary between responsibilities is blurry — methods that straddle both domains — the extraction creates an awkward dependency graph where the two classes constantly call into each other, defeating the purpose of the separation. In such cases, the correct move may be to first simplify the entangled logic with Extract Method, establish cleaner seams, and only then attempt the class-level split.

The refactoring also introduces a relationship that must be managed: construction, lifecycle, and ownership of the new object become explicit concerns. In systems without dependency injection or explicit lifecycle management, this can scatter construction logic. Weigh the clarity gained against the wiring complexity introduced.

## Connections

Extract Class is the inverse of Inline Class. It frequently combines with Move Field and Move Method as the mechanical steps for relocating features. It directly addresses the Large Class smell and the Divergent Change smell by giving each axis of change its own structural home. Data Clumps and Temporary Field also signal extraction opportunities because the clumped data or temporary state often belongs to an unexpressed concept waiting for its own class. The Single Responsibility Principle provides the theoretical grounding: each class should have one reason to change. Extract Subclass serves a related purpose but splits along inheritance lines rather than composition, making it appropriate when the variants share identity rather than merely collaborating. Replace Data Value with Object is a lightweight variant of the same idea — promoting a primitive or simple value into a full-fledged class with its own behavior.

---

*Based on: Refactoring (Fowler, 1999)*
