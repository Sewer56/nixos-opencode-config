### Minimal change
Keep changes minimal; use the smallest viable diff. Avoid broad refactors unless required or requested.

### Plain domain names
Prefer plain code and descriptive, domain-first names for modules, files, types, and functions. Avoid jargon, cleverness, and vague buckets like `utils`, `helpers`, `common`, or `misc` unless already established and intentionally narrow.

### Existing patterns
Prefer existing types, constants, schemas, signatures, and repo patterns.

### Helper size
Inline tiny single-use helpers unless naming improves readability, reuse, or boundaries. Avoid single-implementation abstractions unless needed.

### Obvious control flow
Keep control flow obvious and change sets cohesive.

### Minimal visibility
Keep visibility minimal while preserving required API boundaries.

### Preserve behavior
Preserve behavior unless explicitly changing it.

### Clean changed scope
Remove dead code, unused imports, and newly-unused paths. Avoid debug-only logging, temporary instrumentation, and unnecessary abstractions.

### Line locators
`Lines: ~<start>-<end>` values are approximate (±10 lines); context lines are the authoritative locator.
