### Trust boundaries
Validate identity, authorization, provenance, and ownership when data
or control crosses process, service, tenant, privilege, IPC, plugin,
filesystem, or network boundaries.

### Narrow capabilities
Expose the smallest named capability needed. Avoid generic command/channel invocation, token or secret getters, raw storage primitives, broad filesystem access, and ambient authority when an explicit operation suffices.

### Untrusted input
Validate and normalize untrusted input before it reaches shell, SQL, paths, templates, deserialization, dynamic imports, regular expressions, redirects, or resource allocation. Preserve parameterization and canonicalization boundaries.

### Secrets and sensitive data
Never log, return, persist, cache, or expose secrets beyond their owning boundary. Redact diagnostics; ensure clearing/revocation behavior is complete.

### Failure behavior
Security checks fail closed. Authentication/authorization errors must not fall through to privileged behavior, leak sensitive distinctions, or be converted into success by retries or defaults.

### Dependencies and cryptography
Use established repository mechanisms and maintained libraries. Do not invent cryptography, weaken verification, disable certificate/signature checks, or broaden dependency trust without an explicit approved decision.

### Grounded review
A security finding states the attacker-controlled input or privilege boundary, the reachable path, the missing/incorrect control, and the resulting impact. Generic hardening advice is advisory at most.
