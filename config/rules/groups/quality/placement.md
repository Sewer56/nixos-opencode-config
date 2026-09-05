## Placement

### Ownership
Prefer existing files/modules; create new ones only for material boundary benefits.
Split catch-all files into focused, domain-named modules.
Never collapse modular code into monoliths unless requested.

Keep orchestration in the entrypoint file/module.
Put data-holder models in dedicated `models` modules/directories when supported.
`models` barrel/index files hold wiring and re-exports, not concrete definitions.

Keep enums, newtypes, and value objects with their sole parent type.
Keep non-public helper types local and conversions next to the type.
Never use global `conversions` buckets.

Put shared behavior in the lowest shared package that owns it.
Keep extension, adapter, middleware, and integration packages focused on wiring and package-specific behavior.

When ownership is unclear, use the package others depend on.
Co-locate tests with their module unless the repo has a stronger convention.

### Declaration order
Order declarations most-public to most-private.
Within each visibility tier, put callers before callees: entry points, structs, types, plugins, exports first.

Preserve relative order when priority is equal or dependency unclear.
Do not block whole-file reorder opportunities unless the change leaves a touched declaration clearly out of order.
Do not flag unrelated pre-existing ordering problems.

### Placement ambiguity
Do not guess when placement needs broad semantic inference or repo-wide call-graph reconstruction beyond selected files.
Ask for the missing declaration/call evidence.
