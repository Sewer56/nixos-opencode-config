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
