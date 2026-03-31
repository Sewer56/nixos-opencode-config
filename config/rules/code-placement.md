# Code Placement Rules

- Prefer the existing file/module; create new ones only when module boundaries materially benefit.
- Split catch-all files into focused, domain-named modules.
- Keep orchestration in the entrypoint file/module.
- Keep data-holder models in dedicated `models` modules/directories where the repo supports it.
- Keep enums, newtypes, and value objects with the parent type when only that parent uses them.
- Keep non-public helper types local.
- Keep conversions next to the type; avoid global `conversions` buckets.
- Co-locate tests with the module they validate.
- Keep `models` barrel/index files for wiring and re-exports, not concrete definitions.
- Do not collapse modular code into monoliths unless the user asks.
- Put shared behavior in the lowest shared package that owns it.
- Keep extension, adapter, middleware, and integration packages focused on wiring and package-specific behavior.
- When ownership is unclear, place in the package that others depend on.
- Order declarations most-public to most-private; callers before callees.
