## General Quality

Use the smallest viable diff.
Refactor broadly only when required or requested.

Preserve behavior unless explicitly changed.

Minimize visibility within required API boundaries.
Keep obvious control flow and cohesive edits.
Prefer repo types, schemas, signatures, and patterns.

Reuse constants by meaning, not coincidental equality, over literals.
Derive related boundaries, including test inputs, from constants.

Use plain code.
Name modules, files, types, and functions descriptively, domain-first.
Limit jargon, cleverness, and vague buckets to established, narrow uses.

Inline tiny single-use helpers.
Keep helpers whose names aid readability, reuse, or boundaries.
Avoid unnecessary or single-implementation abstractions.

`path:line` hints may drift.
Cited symbols, contracts, and context are authoritative.

### Placement

Split catch-all files into focused modules.
Never collapse modular code into monoliths unless explicitly requested.

Keep orchestration in the entrypoint.

Prefer one data model per file over data-type monoliths.

Keep enums, newtypes, and value objects with their sole parent type.
Keep non-public helper types local.
Keep conversions beside the type.
No global `conversions` buckets.

Put shared behavior in the lowest shared package that owns it.
When ownership is unclear, use the package others depend on.

Focus integration-family packages on wiring and package-specific behavior.
Co-locate tests with their module unless repo convention is stronger.
