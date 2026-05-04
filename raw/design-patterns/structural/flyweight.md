# Flyweight

## Essence
Flyweight is a structural design pattern that minimizes memory consumption by sharing immutable state across many objects that would otherwise duplicate identical data. Each flyweight object stores only the intrinsic state that is common to all its instances, while extrinsic state (the parts that vary) is stored externally and passed in at call time. A factory manages a pool of existing flyweights, returning a cached instance when the requested intrinsic state already exists.

## Motivation
A text editor represents every character on screen as an object holding its glyph bitmap, font metrics, Unicode code point, and screen position. A document with 200,000 characters allocates 200,000 objects, yet most of the data (the glyph and font metrics) is identical for every occurrence of the same letter in the same font. Only the position varies from one character to the next. Storing the glyph bitmap redundantly in every object wastes megabytes of memory and slows down rendering as the CPU thrashes the cache.

Flyweight separates the shared glyph data (intrinsic state) from the per-character position (extrinsic state). A `GlyphFactory` maintains a cache keyed by font and code point. The first time the letter "e" in 12-point serif is requested, the factory creates a `GlyphFlyweight` holding the bitmap and metrics. Every subsequent request for the same combination returns the cached object. The rendering loop passes the character's screen coordinates as parameters rather than storing them on the flyweight. Two hundred thousand characters now share perhaps a few hundred flyweight objects, reducing memory usage by orders of magnitude.

## Participants
The flyweight interface declares methods that accept extrinsic state as parameters. Concrete flyweight objects store intrinsic state immutably and implement operations using the supplied extrinsic data. An unshared concrete flyweight variant exists for objects whose state cannot be shared and must remain independent. The flyweight factory maintains a registry of created flyweights, checking for existing instances before creating new ones. The client stores extrinsic state, references the appropriate flyweight, and passes the varying context during each operation.

## Application

**Use when:**
- An application creates a very large number of similar objects and available memory is constrained
- Most objects share significant portions of their state that can be extracted and shared
- Extrinsic state can be computed or stored separately without excessive CPU overhead
- Identity-based equality is not required; objects are interchangeable when their intrinsic state matches

**Prefer alternatives when:**
- Object count is modest and memory pressure is not a concern (the pattern's bookkeeping overhead is not justified)
- Extrinsic state is expensive to compute on every call (the CPU-memory tradeoff tips unfavorably)
- Objects are mutable and sharing state would cause unintended side effects between instances

## Consequences
Flyweight can reduce memory consumption dramatically when many objects share identical intrinsic state, which is especially valuable in environments with limited RAM such as mobile devices or embedded systems. The factory's caching logic also speeds up object creation because a cached flyweight is returned immediately without repeating initialization. The trade-off is that extrinsic state must now be stored, computed, or passed explicitly, which adds complexity to the client code and can increase CPU usage if the state is recalculated frequently. The intrinsic state must be immutable; any mutation would corrupt all objects sharing the flyweight. Designers must carefully decide what counts as intrinsic versus extrinsic, and an incorrect partition can negate the memory savings or introduce subtle bugs.

## Relations
Flyweight and Composite work well together because large Composite trees often contain many identical leaf nodes, and making those leaves into flyweights can dramatically reduce the tree's memory footprint. Flyweight is sometimes confused with Singleton, but Singleton enforces exactly one instance of a class while Flyweight permits multiple instances, each distinguished by different intrinsic state; both patterns share a caching mindset. Factory Method or Abstract Factory can construct flyweights, though a dedicated flyweight factory with a registry is more common because it must check for existing instances before creating new ones. State objects can be implemented as Flyweights when many state machine instances share the same state configurations.

---

*Based on: Design Patterns (Gamma, Helm, Johnson, Vlissides, 1994)*
