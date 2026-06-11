### Existing location first
Prefer the existing file/module. Create new modules or files only when module boundaries materially benefit.

### Focused modules
Split catch-all files into focused, domain-named modules. Do not collapse modular code into monoliths unless the user asks.

### Entrypoint and models
Keep orchestration in the entrypoint file/module. Put data-holder models in dedicated `models` modules/directories when the repo supports it. Keep `models` barrel/index files for wiring and re-exports, not concrete definitions.

### Type and conversion ownership
Keep enums, newtypes, and value objects with the parent type when only that parent uses them. Keep non-public helper types local. Keep conversions next to the type; avoid global `conversions` buckets.

### Shared behavior ownership
Put shared behavior in the lowest shared package that owns it. Keep extension, adapter, middleware, and integration packages focused on wiring and package-specific behavior. When ownership is unclear, place code in the package that others depend on.

### Test placement
Co-locate tests with their module unless the repo has a stronger convention.

### Declaration order
Order declarations most-public to most-private. Within each visibility tier, place callers before callees. Place primary entry points, structs, types, plugins, or exports before helpers in call sequence.

### Stable order
When priority is equal or dependency is unclear, preserve existing relative order. Do not block broad whole-file reorder opportunities unless the selected change creates or leaves a touched declaration clearly out of order.

### Placement ambiguity
When placement/order requires broad semantic inference or repo-wide call-graph reconstruction beyond selected files, do not guess; ask for the missing local declaration/call evidence.
