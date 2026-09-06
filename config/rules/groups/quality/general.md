## General Quality

Use the smallest viable diff.
Broad refactors must be required or requested.
Preserve behavior unless explicitly changing it.

Minimize visibility without breaking required API boundaries.
Keep control flow obvious and changes cohesive.
Prefer existing types, constants, schemas, signatures, and repo patterns.

Use plain code.
Give modules, files, types, and functions descriptive domain-first names.
Only use established, intentionally narrow jargon, cleverness, or vague buckets.
Vague buckets include `utils`, `helpers`, `common`, `misc`.

Inline tiny single-use helpers.
Keep helpers when a name aids readability, reuse, or boundaries.
Avoid single-implementation and unnecessary abstractions.

`path:line` hints may drift after repairs.
The cited symbol, contract, and surrounding context are authoritative.
