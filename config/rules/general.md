# General Rules

Use these rules for planning, implementation, and review unless a more specific rules file overrides them.

- Keep changes minimal; prefer the smallest viable diff that fully satisfies the requirement.
- Prefer plain code and names; avoid jargon and cleverness.
- Use descriptive, domain-first names for modules, files, types, and functions.
- Avoid vague bucket names like `utils`, `helpers`, `common`, or `misc` unless they are already established and intentionally narrow.
- Write the least new code that fully satisfies the requirement.
- Prefer existing types, constants, schemas, signatures, and patterns over inventing new ones.
- Reuse existing patterns.
- Inline tiny single-use helpers unless naming them materially improves reuse, readability, or module boundaries.
- Optimize for review: keep control flow obvious and change sets cohesive.
- Keep visibility minimal.
- Within files, order declarations from most public to most private; within each visibility level, define callers before callees (reading order).
- Preserve behavior unless a change is required or explicitly requested.
- Avoid broad refactors unless required or requested.
- Remove dead code, unused imports, and newly-unused paths created by the change.
- Avoid debug/dev-only logging, temporary instrumentation, and unnecessary abstractions.
- Avoid single-implementation interfaces/traits unless there is a concrete need.
