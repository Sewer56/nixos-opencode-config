# Code Placement Rules

- Split catch-all files into focused modules.
- Keep top-level orchestration in the parent module/file entrypoint.
- Keep data-holder models in dedicated model modules.
- Keep non-public helper types local.
- Keep conversions with related type definitions.
- Co-locate tests with the module they validate.
- Put shared behavior in the lowest shared package that owns it.
- If behavior belongs in `core` because every implementation, adapter, or extension should benefit from it, put it in `core`, not in an extension, middleware, or integration package.
- Shared validation, normalization, parsing, and domain contracts belong in shared/core packages when multiple implementations should inherit that behavior.
- Keep extension, adapter, middleware, and integration packages focused on wiring and package-specific behavior.
- If ownership is unclear, prefer the package that other packages depend on, not the package that depends on them.
