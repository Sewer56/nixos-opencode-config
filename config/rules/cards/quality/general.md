### Minimal change
Smallest viable diff; no broad refactors unless required or requested.

### Plain domain names
Plain code, descriptive domain-first names for modules, files, types,
and functions; no jargon, cleverness, or vague buckets (`utils`,
`helpers`, `common`, `misc`) unless established and intentionally narrow.

### Existing patterns
Prefer existing types, constants, schemas, signatures, and repo
patterns.

### Helper size
Inline tiny single-use helpers unless a name improves readability,
reuse, or boundaries; no single-implementation abstractions.

### Obvious control flow
Keep control flow obvious, change sets cohesive.

### Minimal visibility
Keep visibility minimal while preserving required API boundaries.

### Preserve behavior
Preserve behavior unless explicitly changing it.

### Clean changed scope
Remove dead code, unused imports, and newly-unused paths; avoid
debug-only logging, temporary instrumentation, and unnecessary
abstractions.

### Review locators
`path:line` references are navigation hints and may drift after repairs; the cited symbol, contract, and surrounding context are authoritative.
