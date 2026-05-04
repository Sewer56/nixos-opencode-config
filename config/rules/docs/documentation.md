## Documentation Rules

Use these rules when writing or updating documentation in changed scope.

### Required Docs
- Public APIs (`pub`, `pub(crate)`): purpose and parameters.
- Non-trivial public APIs: add returns, failure behavior, examples when helpful.
- Non-trivial private APIs: purpose plus non-obvious parameters, returns, side effects, invariants.
- Trivial private APIs: no full docs needed.
- New/changed modules: top-level docs with purpose and usage.

### Placement & Maintenance
- Package docs cover import/usage shape; in-code docs cover exported symbols — update both when both exist.
- If examples requested: add to in-code API docs, not just package docs.
- If no native module docs: use nearest file-level doc block.
- Update existing docs when behavior changes; remove only if incorrect or inapplicable.
- When moving or renaming documented items, preserve or replace the affected docs.
- If a change alters module/file boundaries, refresh boundary docs.

### Style
- Lead with a one-sentence purpose in plain language.
- Section order: summary, `Arguments`, `Returns`, `Errors`, `Examples`.
- Use focused headings (`Arguments`, `Returns`, `Errors`, `Examples`, `Usage`, `Public API`); list entrypoints by role under `Public API`.
- Prefer goal-oriented phrasing: "split paths" not "materialize path groups", "resolve early" not "JIT resolution".
- Reference symbols using language convention for doc links (Rust: `[`TypeName`]`). Use doc-link syntax for type/variant references where the language supports it; plain backticks only for value expressions (`path`, `count`).
- Always include a language tag on fenced code blocks; never use bare `ignore` fences.
- Keep examples practical and minimal.

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
/// # Errors
/// - Returns [`Error::InvalidPath`] when `paths` contains entries not valid
///   relative to the installer root.
///
/// # Examples
/// ```rust
/// let paths = vec!["Pack/".to_string(), "Pack/file.txt".to_string()];
/// let groups = split_paths_by_kind(paths);
/// assert_eq!(groups.files, vec!["Pack/file.txt"]);
/// assert_eq!(groups.directories, vec!["Pack"]);
/// ```
/// 
/// [`Error::InvalidPath`]: crate::error::Error::InvalidPath
pub fn split_paths_by_kind(paths: Vec<String>) -> PathGroups { ... }
```

### Inline Readability Comments
- In non-trivial function bodies, add a short inline comment at each logical step describing intent — a reader scanning only comments should understand the function.
- Skip obvious steps; prefer `// Strip the search-root prefix to get a relative path` over `// check if ft is dir`.

### Review Blocking Criteria
- Docs must not contradict implementation.
- In machine plans: docs must appear in relevant snippet/diff; generic `update docs` note is insufficient.
- Do not backfill untouched legacy files solely for docs.
