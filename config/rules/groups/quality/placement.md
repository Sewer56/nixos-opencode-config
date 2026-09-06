## Placement

Split catch-all files into focused, domain-named modules.
Never collapse modular code into monoliths unless explicitly requested.
Keep orchestration in the entrypoint.

Avoid data-type monoliths by preferring one data model per file.

Keep enums, newtypes, and value objects with their sole parent type.
Keep non-public helper types local.
Keep conversions beside the type.
No global `conversions` buckets.

Put shared behavior in the lowest shared package that owns it.
When ownership is unclear, use the package others depend on.

Focus integration-family packages on wiring and package-specific behavior.
Co-locate tests with their module unless repo convention is stronger.
