## Code Placement Rules

Use these rules when adding, moving, or renaming modules, files, types, or functions.

### Placement
- Prefer the existing file/module; create new ones only when module boundaries materially benefit.
- Split catch-all files into focused, domain-named modules.
- Keep orchestration in the entrypoint file/module.
- Keep data-holder models in dedicated `models` modules/directories where the repo supports it.
- Keep enums, newtypes, and value objects with the parent type when only that parent uses them.
- Keep non-public helper types local.
- Keep conversions next to the type; avoid global `conversions` buckets.
- Co-locate tests with their module.
- Keep `models` barrel/index files for wiring and re-exports, not concrete definitions.
- Do not collapse modular code into monoliths unless the user asks.
- Put shared behavior in the lowest shared package that owns it.
- Keep extension, adapter, middleware, and integration packages focused on wiring and package-specific behavior.
- When ownership is unclear, place in the package that others depend on.

### Ordering
- Order declarations most-public to most-private.
- Within each visibility tier, place callers before callees (reading order).
- Place the structs and entry point first; then helpers in call sequence.

### Example

```text
# Prefer
src/package/mod.rs       # re-exports only
src/package/installer.rs # public install()
src/package/fetch.rs     # private fetch_package()

# Avoid
src/utils.rs   # fetch_package(), format_path(), parse_version() — unrelated
src/helpers.rs # mixed helpers for different domains
```
