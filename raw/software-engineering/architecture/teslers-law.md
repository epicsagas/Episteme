# Tesler's Law

## Statement
Tesler's Law, also called the Law of Conservation of Complexity, holds that every application or system contains an irreducible core of complexity that cannot be eliminated — only relocated from one part of the system to another. Making a user interface simpler typically pushes complexity into the backend, into configuration, into build tooling, or into the mental model the developer must maintain. The total quantity of complexity is conserved; the design decision is where that complexity resides and who bears its cost.

## Origin
Larry Tesler, a computer scientist instrumental in the development of the graphical user interface at Xerox PARC and later Apple, articulated this principle during his research on human-computer interaction in the 1980s. Tesler's work on modeless editing, cut-and-paste interactions, and direct manipulation interfaces led him to observe that simplification in one dimension always transfers effort to another. The law was formalized in discussions within the design community and has since become a foundational concept in interaction design and systems engineering.

## Software Implications
The law manifests wherever an engineer tries to make something "just work" for the end user. A deployment platform that reduces the interface to a single button must internally contain sophisticated logic for environment detection, dependency resolution, service discovery, and rollback orchestration. That complexity has not disappeared; it now lives in the platform team's codebase, on-call rotations, and incident postmortems. The user is shielded, but the total system complexity is unchanged.

Low-code and no-code tools illustrate the trade vividly. They promise to eliminate programming complexity by replacing code with visual builders, but the complexity resurfaces as conditional logic buried in dropdown menus, data mapping rules hidden in property panels, and debugging sessions that require understanding both the visual metaphor and the generated code beneath it. When the visual abstraction reaches its limits, users encounter the conserved complexity all at once, often without the vocabulary to describe what has gone wrong.

API design is another arena where Tesler's Law operates. A library that exposes a single function with sensible defaults conceals complexity in the parameter resolution logic, the default selection algorithm, and the compatibility matrix across supported platforms. A library that exposes every knob explicitly places the complexity on the caller. Neither approach reduces total complexity; they differ only in where the burden falls.

## Practical Guidance
- Before simplifying a surface, identify where the displaced complexity will land and verify that the receiving team or component is equipped to handle it.
- Make complexity visible to the people best positioned to manage it; hiding it from everyone often means no one is prepared when it surfaces.
- Choose consciously between opinionated defaults (complexity in the platform) and explicit configuration (complexity on the caller), and document the rationale.
- When users report that a system is "too simple" or "too magical," the conserved complexity may be leaking in ways they cannot articulate — treat that feedback as a design signal.

## Common Misreadings
A common oversimplification reads Tesler's Law as proof that all design effort is futile because complexity is inescapable. The law says nothing about the quality of complexity, only its quantity. Well-placed complexity is learnable, debuggable, and proportionate to the task; poorly placed complexity is opaque, surprising, and brittle. Relocating complexity well is the essence of good design. Another misreading treats the law as a precise thermodynamic conservation rule; in practice, complexity can grow or shrink slightly depending on how a problem is reframed, but the core insight — that elimination in one place forces appearance elsewhere — holds robustly.

## Interactions
The Law of Leaky Abstractions explains one mechanism by which conserved complexity resurfaces: when the layer that absorbed the complexity cannot fully contain it, details bleed through to the caller. Hyrum's Law means that once complexity has been placed somewhere, the behaviors it produces become depended upon, making future relocation expensive. Gall's Law intersects with Tesler's Law when a simple system evolves into a complex one: the conserved complexity that was manageable in the small system becomes distributed across many new components as the system grows, and careful placement determines whether the result is navigable or chaotic. Zawinski's Law drives the ongoing accumulation of complexity that Tesler's Law says must be allocated somewhere, creating persistent pressure on architectural boundaries.

---

*Based on: Tesler, HCI research*
