# Builder

## Essence
Builder is a creational design pattern that constructs complex objects through a sequence of discrete steps, letting callers assemble different representations without altering the construction algorithm. Rather than passing a sprawling list of parameters to a single constructor, the client invokes named methods on a builder object, each method configuring one aspect of the final product. The builder defers instantiation until every step completes, then returns the fully assembled object.

## Motivation
Imagine a report-generation engine that produces financial statements in PDF, HTML, and spreadsheet formats. Every report shares the same assembly stages: build a header with company metadata, append a summary table, attach detailed line items, and render footer disclaimers. A single constructor would need parameters for every configurable option across all three formats, most of which would be irrelevant for any given run. Subclassing a base `Report` for each format multiplies class count, and the logic for sequencing stages would still be duplicated across those subclasses.

Builder resolves this by placing each format's rendering details inside its own builder class. A `PdfReportBuilder` knows how to lay out headers in PDF primitives, while an `HtmlReportBuilder` wraps the same data in markup tags. An optional director class encodes the assembly sequence (header, summary, details, footer) and drives any builder through those stages in order. Adding a `CsvReportBuilder` later requires no changes to the director or to client code that calls it.

## Participants
A builder interface declares the construction steps every concrete builder must support: build header, build summary, build details, build footer, and retrieve the result. Each concrete builder implements those steps, accumulating state internally and producing a finished product when asked. The product itself carries no requirement for a shared base class; each builder can return a different type. An optional director encapsulates a reusable assembly sequence, calling builder methods in a fixed order. The client creates the appropriate builder, optionally passes it to a director, then retrieves the finished product from the builder.

## Application

**Use when:**
- An object requires many optional or conditional parameters that make telescoping constructors unreadable
- Different representations of a product share most construction steps but differ in a few details
- Construction involves recursive or tree-shaped structures where steps may be repeated at varying depths (Composite hierarchies)
- The assembly algorithm should be reusable across multiple product variations

**Prefer alternatives when:**
- The product has only a handful of mandatory fields (a plain constructor or Factory Method is clearer)
- The caller needs the object immediately without step-by-step assembly
- Variants differ only in data values, not in structural configuration (Prototype may be simpler)

## Consequences
Builder distributes construction logic across focused methods, which improves readability and makes each step independently testable. Clients can skip optional steps or invoke them in a custom order when no director is used, giving fine-grained control over the final product. The trade-off is additional classes: every product representation needs its own concrete builder, and an interface plus optional director add further types. For objects with only a few fields, this ceremony is unnecessary overhead. The pattern also shines when construction must be incremental: a client can store a partially configured builder, resume assembly later, or even serialize the builder state for deferred processing. Debugging is easier because each step can log or validate its input before mutation occurs.

## Relations
Factory Method produces a single object in one call, while Builder spreads creation across many calls. Abstract Factory returns families of products immediately, whereas Builder lets the caller interleave configuration steps before requesting the result. Builder and Composite are natural partners: a builder can recursively assemble a tree of composite nodes, calling itself on child elements. Bridge shares a structural resemblance because a director acts as the abstraction and concrete builders serve as implementations, though Bridge addresses orthogonal hierarchies rather than construction sequencing. Prototype can supply the initial state that a builder modifies, combining fast cloning with incremental customization. Concrete builders are sometimes implemented as Singleton when the system reuses the same builder instance repeatedly.

---

*Based on: Design Patterns (Gamma, Helm, Johnson, Vlissides, 1994)*
