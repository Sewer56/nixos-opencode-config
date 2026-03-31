## Documentation Rules

## Scope
- In changed scope, document public APIs and exports (Rust: `pub` and `pub(crate)` items).
- In changed scope, document non-trivial private APIs.
- When both package-level docs (`README.md` or nearest usage doc) and in-code API docs exist in changed scope, update both.
- If a change materially alters a module/file boundary, refresh module/file docs.
- Update existing documentation as needed.
- Do not remove existing documentation unless it is incorrect or no longer applies.
- When moving or renaming documented items, preserve or replace the affected docs.

## Required Docs
- Public APIs/exports: purpose. Document parameters.
- Non-trivial public APIs: add returns, failure behavior, examples when helpful.
- Non-trivial private APIs: purpose plus non-obvious parameters, returns, side effects, invariants.
- Trivial private APIs: no full docs needed.
- If examples requested: add to in-code API docs, not just package docs.
- New/changed modules: top-level docs with purpose and usage.
- Package docs: import/usage shape; in-code docs: exported symbols.
- If no native module docs: use nearest file-level doc block.
- Use focused headings: `Public API`, `Arguments`, `Returns`, `Examples`, `Usage`, `Errors`, `Validation`, `Identifier Format`, `Precedence`.
- `Public API`: list public entrypoints/types by role.
- Reference symbols using language convention (Rust: `[`TypeName`]`).
- Never use `ignore` fences.

## Style
- Lead with a one-sentence purpose in plain language.
- For sectioned function and method docs, use this order: short summary, `Arguments`, `Returns`, then `Examples`.
- Prefer goal-oriented phrasing over implementation terms.
- Avoid jargon: no "materialization", "JIT", "framework-agnostic", "deterministic resolution", etc.
- Keep examples practical and minimal.
- When the doc format supports fenced examples, include a language tag (e.g., `rust`).
- Dense but accessible.

### API Doc Example

```rust
/// Split raw installer paths into files and explicit directories.
///
/// # Arguments
/// - `paths`: Raw installer-relative paths where trailing separators mark directories.
///
/// # Returns
/// - [`PathGroups`]: Split file paths and explicit directory paths.
///
/// # Examples
/// ```rust
/// let paths = vec!["Pack/".to_string(), "Pack/file.txt".to_string()];
/// let groups = split_paths_by_kind(paths);
/// assert_eq!(groups.files, vec!["Pack/file.txt"]);
/// assert_eq!(groups.directories, vec!["Pack"]);
/// ```
pub fn split_paths_by_kind(paths: Vec<String>) -> PathGroups {
    PathGroups { files: Vec::new(), directories: Vec::new() }
}
```

## Review Bar
- Missing required docs is blocking.
- When both package-level and in-code docs are in scope: missing either side is blocking.
- Missing docs for non-trivial private APIs in changed scope is blocking.
- Docs must not contradict implementation.
- Keep docs dense, not skeletal.
- If examples explicitly requested: README-only is insufficient.
- In machine plans: docs must appear in relevant snippet/diff; generic `update docs` note is insufficient.
- Do not backfill untouched legacy files solely for docs.
