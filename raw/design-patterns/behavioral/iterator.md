# Iterator

## Essence
Iterator is a behavioral design pattern that provides a uniform interface for traversing the elements of an aggregate object without exposing its underlying representation. Whether the collection is an array, a linked list, a binary tree, or a graph, client code accesses elements through the same traversal contract. Multiple independent iterators can navigate the same collection simultaneously, each maintaining its own position and progress state.

## Motivation
Picture a music streaming service whose library stores tracks in different structures depending on context: playlists use arrays, user favorites use a hash set, and genre hierarchies use trees. The user interface must render track listings regardless of storage format, and shuffle or reverse-ordered playback must work across all of them. Embedding traversal logic for every storage variant into the UI layer would scatter collection-specific code throughout the application and lock the UI to internal data decisions.

Iterator solves this by extracting navigation into dedicated objects. Each collection produces iterators tailored to its structure, yet all iterators conform to a shared interface. The UI layer calls next and hasNext without caring whether elements come from an array index, a hash bucket traversal, or a tree walk.

## Participants
An Iterator interface declares the core traversal operations: fetching the next element, checking whether more elements remain, and optionally resetting to the start. Concrete iterators implement these operations for a specific aggregate structure, tracking traversal position internally. The Aggregate interface defines a factory method that returns an iterator compatible with the collection. Concrete aggregates instantiate and return the appropriate iterator type. The client retrieves an iterator from the aggregate and uses only the iterator interface to walk the elements, never reaching into the collection directly.

## Application

**Use when:**
- client code must traverse diverse collection types through a single interface
- a collection needs to support multiple traversal strategies such as depth-first, breadth-first, or in-order
- you want to isolate traversal algorithms from the data structure's primary responsibility of storage

**Prefer alternatives when:**
- the collection is a simple array or list and language-level iteration constructs such as for-each loops suffice
- only one traversal order exists, the structure is trivial, and adding an iterator class would add unnecessary indirection

## Consequences
Iterator upholds the Single Responsibility Principle by pulling traversal logic out of collection classes and the Open/Closed Principle by allowing new traversal strategies without modifying existing collections. Supporting multiple simultaneous traversals of the same collection is a direct benefit of isolating position state in each iterator object. The trade-off is that creating iterator objects incurs a small allocation cost per traversal, and for performance-critical inner loops the overhead may matter. Debugging can also become slightly harder because iteration state is distributed across a separate object rather than visible as a loop variable on the stack.

## Relations
Iterator works hand in hand with Composite, providing a clean way to walk recursive tree structures without exposing node internals. Factory Method is often employed by aggregate classes to produce the correct iterator subtype. Memento can capture an iterator's position for checkpoint-and-resume scenarios. Visitor complements Iterator by performing type-specific operations on heterogeneous elements during traversal, whereas Iterator handles only the navigation concern.

---

*Based on: Design Patterns (Gamma, Helm, Johnson, Vlissides, 1994)*
