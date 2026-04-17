## General Rules

Use these rules for planning, implementation, and review unless a more specific rules file overrides them.

### Rules

- Keep changes minimal; use the smallest viable diff.
- Prefer plain code and names; avoid jargon and cleverness.
- Use descriptive, domain-first names for modules, files, types, and functions.
- Avoid vague bucket names like `utils`, `helpers`, `common`, or `misc` unless already established and intentionally narrow.
- Prefer existing types, constants, schemas, signatures, and patterns.
- Inline tiny single-use helpers unless naming improves readability, reuse, or boundaries.
- Keep control flow obvious and change sets cohesive.
- Keep visibility minimal.
- Preserve behavior unless explicitly changing it.
- Avoid broad refactors unless required or requested.
- Remove dead code, unused imports, and newly-unused paths.
- Avoid debug-only logging, temporary instrumentation, and unnecessary abstractions.
- Avoid single-implementation abstractions (interfaces, traits, protocols) unless needed.

### Example

```diff
- fn handle(d: &Data) { ... }
+ fn install_package(pkg: &Package) { ... }

# Prefer src/package/installer.rs over src/utils/mod.rs
```

Line numbers in diff hunks are approximate; context lines are the authoritative locator.
