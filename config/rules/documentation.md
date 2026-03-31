# Documentation Rules

## Scope
- In changed scope, document public APIs and exports (Rust: `pub` and `pub(crate)` items).
- In changed scope, document non-trivial private APIs.
- When both package-level docs (`README.md` or nearest usage doc) and in-code API docs exist in changed scope, update both.
- If a change materially alters a module/file boundary, refresh module/file docs.
- Update existing documentation as needed.
- Do not remove existing documentation unless it is incorrect or no longer applies.
- When moving or renaming documented items, preserve or replace the affected docs.

## Required Docs
- Public APIs and exports: brief purpose. Document parameters on public APIs.
- Non-trivial public APIs: also document returns, failure behavior, and examples when requested or helpful.
- Non-trivial private APIs: brief purpose plus any non-obvious parameters, returns, side effects, or invariants.
- Trivial private APIs do not need full API docs.
- If examples are requested, add them to in-code API docs, not only package-level docs.
- New or materially changed modules/files: top-level docs with purpose and usage context.
- Package-level docs cover import/usage shape; in-code docs cover exported symbols.
- If the language lacks native module docs, use the nearest file-level doc block/comment.
- Add focused headings when useful: `Public API`, `Arguments`, `Returns`, `Examples`, `Usage`, `Errors`, `Validation`, `Identifier Format`, `Precedence`.
- `Public API` lists public entrypoints/types by role.
- Reference linked symbols using the language's doc convention (e.g., Rust: `[`TypeName`]`).
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

Example for non-trivial public APIs:

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
    PathGroups {
        files: Vec::new(),
        directories: Vec::new(),
    }
}
```

## Review Bar
- Missing required docs is blocking.
- Docs must not contradict implementation.
- Keep docs dense, not skeletal.
- If examples were explicitly requested, README-only examples are insufficient.
- In machine plans, docs must appear in the relevant implementation snippet/diff; a generic `update docs` note is insufficient.
- Do not backfill untouched legacy files solely for docs.
