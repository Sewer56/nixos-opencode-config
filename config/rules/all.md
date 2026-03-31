# Plan Content Rules

- No placeholders (`...`, `TODO`, comment-only test bodies).
- No undefined helpers/types/symbols in snippets.
- Insertions use normal code blocks with `Insert at: <anchor> (~start-end)`.
- Edits/removals use `diff` blocks; deletions include `Remove lines: ~start-end`.
- Import changes use a dedicated import `diff` block.
- If layout changes, include target tree and migration order.

# General Rules

Use these rules for planning, implementation, and review unless a more specific rules file overrides them.

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
- Add inline comments for non-obvious logic.

# Performance Rules

- Prefer the highest-performance correct implementation.
- Then simplify for readability and reviewability, but never trade meaningful performance for brevity or superficial simplicity.

# Testing Rules

Use these rules when the task requires tests.

- Cover all new code with tests.
- Avoid duplicate coverage and setup; do not restate what an existing test already proves.
- Reuse existing test helpers; extract shared helpers only when they reduce repetition or clarify setup across multiple tests.
- Keep tests deterministic; avoid real I/O, time, and network unless controlled, seeded, or frozen.
- When one behavior needs multiple similar cases, follow `test-parameterization.md` for naming, labels, and case structure.

# Test Parameterization Rules

Use these rules when a single behavior needs multiple similar test cases.

- Prefer parameterized tests for multiple inputs on the same logic path; use separate tests only when setup, assertions, or failure modes differ.
- Give each case a descriptive name and parameter labeling style (e.g. `empty_input_returns_zero`); avoid generic names like `case_1`.
- Keep argument order stable: primary input -> mode/flags -> expected output.
- Label parameters with short comments only when non-obvious.
- Keep labels aligned where practical.
- If inline labels become too long, move labels above the case.
- Comment non-obvious setup or assertions inline.
- Keep tests human-friendly and around 80-100 characters per line.
- For Rust: prefer `rstest` with `#[case::name(...)]` and aligned parameters/comments.

## Style Reference

```rust
/// Verifies line truncation in formatted output.
#[rstest]
#[case::with_line_numbers(
    6,           // max_len: truncate "abcdefghij" (10 chars) to 6
    true,        // with_line_numbers: yes, shows "L1: " prefix
    "L1: abc..." // expected: truncated with line number prefix
)]
#[case::without_line_numbers(
    4,        // max_len: truncate to 4 chars
    false,    // with_line_numbers: no prefix
    "  a..."  // expected: truncated without prefix
)]
fn grep_format_handles_line_truncation(
    #[case] max_len: usize,
    #[case] with_line_numbers: bool,
    #[case] expected: &str,
) {
    // Keep setup short; comment only non-obvious assertions.
}
```

# Code Placement Rules

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
- Order declarations most-public to most-private; callers before callees.

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

# Orchestration Plan Rules

For orchestrator workflow -PLAN.md files.

- Map each requirement to its implementation step(s) and test step(s) or assertion(s).
- Include `## Requirement Trace Matrix` with requirement, implementation step ref(s), test step ref(s), and acceptance criteria.
- Keep `## External Symbols` current.
- In `## Implementation Steps`, each step includes `Action`, `Anchor`, `Lines` (approx), and `Order` (if needed).

# Orchestration Revision Rules

For orchestrator plan review, revision, and review ledger handling.

- Preserve issue IDs across revisions when root cause is unchanged.
- Include `acceptance_criteria` for each open issue ID (short, testable closure condition).
- Point to changed implementation/test sections that close each issue.
- Include `## Revision Impact Table` on revisions (changed hunk/step -> affected requirement(s) -> affected test(s)).
- Do not reopen resolved issues without new evidence.
