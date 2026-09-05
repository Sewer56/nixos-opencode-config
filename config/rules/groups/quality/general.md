## General Quality

Use the smallest viable diff; no broad refactors unless required or requested.
Preserve behavior unless explicitly changing it.

Keep visibility minimal while preserving required API boundaries.
Keep control flow obvious and change sets cohesive.
Prefer existing types, constants, schemas, signatures, and repo patterns.

Use plain code and descriptive domain-first module, file, type, and function names.
Avoid jargon, cleverness, and vague buckets (`utils`, `helpers`, `common`, `misc`) unless established and intentionally narrow.

Inline tiny single-use helpers unless a name improves readability, reuse, or boundaries.
Avoid single-implementation abstractions.

Remove dead code, unused imports, and newly-unused paths in changed scope.
Avoid debug-only logging, temporary instrumentation, and unnecessary abstractions.

`path:line` references are navigation hints and may drift after repairs.
The cited symbol, contract, and surrounding context are authoritative.
