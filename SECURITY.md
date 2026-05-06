# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | ✅ Yes    |

## Reporting a Vulnerability

Please **do not** open a public GitHub issue for security vulnerabilities.

Instead, report them privately:

- **Email:** [Open a private advisory](https://github.com/epicsagas/Syntagma/security/advisories/new) on GitHub
- **Response time:** We aim to acknowledge within 48 hours and resolve within 14 days

## Scope

Syntagma runs entirely offline as a local binary — it makes no external network calls in normal operation.

In-scope:
- Command injection via CLI arguments
- Path traversal in file analysis
- SQL injection in the local SQLite layer
- Unsafe deserialization of knowledge graph data

Out-of-scope:
- Vulnerabilities in the user's Rust toolchain or OS
- Issues requiring physical access to the machine
