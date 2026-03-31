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
- Inline tiny single-use helpers unless naming improves readability, reuse, or module boundaries.
- Keep control flow obvious and change sets cohesive.
- Keep visibility minimal.
- Preserve behavior unless explicitly changing it.
- Avoid broad refactors unless required or requested.
- Remove dead code, unused imports, and newly-unused paths.
- Avoid debug-only logging, temporary instrumentation, and unnecessary abstractions.
- Avoid single-implementation abstractions (interfaces, traits, protocols) unless a concrete need exists.
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
- For Rust, prefer `rstest` with `#[case::name(...)]` and aligned labeled parameters/comments.

## Style Reference

```rust
/// Verifies that line truncation in formatted output behaves correctly for
/// different line lengths and line number settings.
#[rstest]
#[case::with_line_numbers_short(
    6,           // max_len: line "abcdefghij" (10 chars) truncated to 6
    true,        // with_line_numbers: yes, shows "L1: " prefix
    "L1: abc..." // expected: truncated with line number prefix
)]
#[case::without_line_numbers_short(
    4,        // max_len: line truncated to 4 chars
    false,    // with_line_numbers: no prefix
    "  a..."  // expected: truncated without line number prefix
)]
#[case::no_truncation_when_fits(
    200,             // max_len: larger than line length (10 chars)
    true,            // with_line_numbers: yes
    "L1: abcdefghij" // expected: full line preserved, no truncation
)]
#[case::exact_boundary_no_truncation(
    10,              // max_len: exactly matches line length (10 chars)
    true,            // with_line_numbers: yes
    "L1: abcdefghij" // expected: full line preserved, boundary not exceeded
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
- Co-locate tests with the module they validate.
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
