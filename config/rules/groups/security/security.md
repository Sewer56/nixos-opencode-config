## Security

### Trust boundaries
Validate identity, authorization, provenance, and ownership across trust boundaries.
These include process, service, tenant, privilege, IPC, plugin, filesystem, and network boundaries.

Expose the smallest named capability needed.
Avoid generic command/channel invocation, token/secret getters, raw storage, broad filesystem access, and ambient authority when an explicit operation suffices.

### Untrusted input
Validate and normalize untrusted input before shell, SQL, paths, templates, deserialization, dynamic imports, regular expressions, redirects, or resource allocation.
Preserve parameterization and canonicalization boundaries.

### Secrets and failure behavior
Never log, return, persist, cache, or expose secrets beyond their owning boundary.
Redact diagnostics; ensure complete clearing/revocation behavior.

Security checks fail closed.
Auth errors must not reach privileged behavior, leak sensitive distinctions, or become success through retries/defaults.

### Dependencies and cryptography
Use established repository mechanisms and maintained libraries.
Never invent cryptography, weaken verification, disable certificate/signature checks, or broaden dependency trust without an explicit approved decision.

### Grounded review
A security finding states attacker-controlled input or privilege boundary, reachable path, missing/incorrect control, and impact.
Generic hardening advice is advisory at most.
