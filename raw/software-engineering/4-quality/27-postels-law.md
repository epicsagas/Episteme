# Postel's Law

## Statement
Postel's Law requires systems to be strict in what they emit and tolerant in what they accept. This robustness principle ensures that independently developed components can interoperate despite minor deviations from shared specifications.

## Origin
Jon Postel articulated this principle in RFC 793, the Transmission Control Protocol specification published in 1981. The directive appeared in the context of early internet protocols, where implementations varied across vendors and backward compatibility was essential for network growth. The guideline proved so broadly applicable that it became a foundational tenet of API design, parsing, and interface contract engineering across all of software.

## Software Implications
An HTTP server that responds with precisely formatted headers and status codes demonstrates conservative output. The same server should accept requests with extra whitespace, unknown headers, or body fields it does not use, demonstrating liberal input. A JSON parser that strictly serializes output according to RFC 8259 but tolerates trailing commas or single-quoted strings on input embodies the same principle. The asymmetry preserves interoperability: strict output gives consumers a reliable contract, while lenient input accommodates diverse producers without breaking the integration.

The law has particular force in versioned APIs. When a service introduces a new field in its response, clients following Postel's Law simply ignore the unknown field. When a client sends a deprecated field, the server that accepts it gracefully avoids forcing a coordinated migration. This tolerance extends system lifetimes and reduces coupling between release cycles.

However, lenient input handling must never compromise security. Accepting malformed input for compatibility is different from executing unvalidated input. The boundary is parsing versus processing: consume broadly, but validate strictly before acting.

## Practical Guidance
- Forward-compatible APIs should ignore unknown fields in requests and include new fields only when clients need them.
- Parsers should normalize variant input formats internally rather than reject them outright.
- Document the strict output contract clearly so consumers know exactly what to rely on.
- Log deviations in received input at warning level to surface integration drift early.

## Common Misreadings
A frequent misreading equates "liberal in what you accept" with "accept anything," including malicious payloads. The law mandates tolerance of benign variation, not abandonment of validation. Accepting a date in multiple formats is compliant; executing raw SQL from user input is negligent. Another error is applying leniency only on one side: a system that sends sloppy output while rejecting minor input variations violates the law's symmetry. The most costly misunderstanding is using the law to justify never deprecating anything, leading to indefinitely growing compatibility surfaces.

## Interactions
Postel's Law directly supports Murphy's Law by building tolerance for the malformed inputs that Murphy guarantees will arrive. It reduces Technical Debt by lowering the coordination cost of API version upgrades. The principle complements Lehman's Laws because evolving systems need flexible interfaces to adapt without breaking existing consumers. The Testing Pyramid applies here: unit tests should verify strict output, while integration tests should exercise tolerant input parsing.

---

*Based on: Postel, IETF TCP specification (1981)*
