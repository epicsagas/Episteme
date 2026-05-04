# Hyrum's Law

## Statement
Hyrum's Law asserts that once an API accumulates enough consumers, every observable behavior of that interface will be relied upon by someone, regardless of whether it was documented as a guarantee. The timing of a response, the wording of an error message, the order of items in a collection returned by an internal helper — all become implicit contracts the moment a user's code depends on them. Even behaviors the author considers accidental or temporary transform into load-bearing dependencies at scale.

## Origin
Hyrum Wright, a software engineer at Google, articulated this principle while reflecting on the challenges of maintaining large-scale APIs shared across thousands of internal teams. It appears in "Software Engineering at Google," co-authored with Titus Winters and Tom Manshreck, where it serves as a cautionary lens for anyone designing public or widely-used interfaces. Wright named it after himself to emphasize that he was describing an empirical observation, not prescribing a rule.

## Software Implications
In practice, Hyrum's Law means that library and platform maintainers cannot safely change anything users can observe, not just things the documentation promises. A hash function that produces a different iteration order after a patch upgrade will break tests that assert on exact output. A REST endpoint that begins returning dates in a different format will silently corrupt downstream data pipelines. The surface area of an interface extends far beyond its declared contract.

This dynamic shapes versioning strategy. Semantic versioning and deprecation workflows exist largely to manage Hyrum's Law effects: they give consumers time to adapt before an observable behavior changes. Teams that skip deprecation and push breaking changes under the banner of "we never promised that" erode trust and force downstream engineers into defensive workarounds such as pinning exact dependency versions or vendoring entire libraries.

Testing practices also shift under Hyrum's Law. Well-designed test suites verify the documented contract and intentionally avoid coupling to incidental details like log message wording or internal data representations. When tests depend on observable but undocumented behavior, refactoring becomes hazardous and the cost of change escalates sharply.

## Practical Guidance
- Audit your public interfaces for behaviors that are observable but not documented, and decide explicitly whether each is a guarantee or an implementation detail.
- Hide implementation details behind stable facades; expose only what you are willing to maintain forever.
- Adopt semantic versioning and deprecation cycles so consumers can plan for behavioral changes.
- Write tests that assert on contract semantics, not on incidental output that may change.

## Common Misreadings
A frequent misinterpretation treats Hyrum's Law as an argument against changing anything ever, effectively freezing APIs in amber. The law does not demand stasis; it demands awareness. Change is healthy, but invisible or unannounced change is corrosive. Another error is assuming the law applies only to formally published APIs. In reality, any shared module, internal endpoint, or command-line flag with multiple consumers is subject to the same dynamics.

## Interactions
Hyrum's Law intensifies the consequences described by the Law of Leaky Abstractions because leaked details become additional observable behaviors that users latch onto. It reinforces the stability concerns underlying Zawinski's Law: as a program accumulates features, its observable surface area grows, giving more footholds for implicit dependencies. The Second System Effect compounds Hyrum's Law when a rewrite changes incidental behaviors that the original system's consumers had come to rely on. Conversely, Gall's Law offers a mitigation path — evolving a system incrementally rather than replacing it wholesale keeps the set of observable behaviors relatively stable.

---

*Based on: Winters, Manshreck & Wright, "Software Engineering at Google" (O'Reilly, 2020)*
