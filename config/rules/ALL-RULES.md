# All Rules

This file combines the shared rules and the orchestrator-specific rules in one place for workflows that need the full ruleset in one read.

## Plan Content
- No placeholders (`...`, `TODO`, comment-only test bodies).
- No undefined helpers/types/symbols in snippets.
- Insertions use normal code blocks with `Insert at: <anchor> (~start-end)`.
- Edits/removals use `diff` blocks; deletions include `Remove lines: ~start-end`.
- Import changes use a dedicated import `diff` block.
- If layout changes, include target tree and migration order.

## General
- Keep changes minimal.
- Prefer plain code and names; avoid jargon and cleverness.
- Write the least new code that fully satisfies the requirement.
- Reuse existing patterns.
- Keep visibility minimal.
- Within files, order declarations from most public to most private; within each visibility level, define callers before callees (reading order).
- Avoid broad refactors unless required or requested.
- Avoid dead code, debug/dev-only logging, and unnecessary abstractions.

## Performance
- Prefer the highest-performance correct implementation that still keeps the code readable.
- Simplify only after performance is preserved; do not give up meaningful performance just to make the code look shorter or superficially simpler.

## Test Parameterization
- Avoid duplicate tests and test helpers.
- Prefer parameterized tests when multiple inputs exercise the same logic path; keep separate tests only when setup, assertions, or failure modes differ.
- When planning parameterized tests, include representative case naming and parameter labeling style (for example `empty_input_returns_zero`).
- Give each case a descriptive name; avoid generic names like `case_1`.
- Keep argument order stable: primary input -> mode/flags -> expected output.
- Label parameters with short plain-English comments only when the meaning is non-obvious.
- Keep labels aligned where practical.
- If inline labels become too long, move labels above the case.
- Add occasional in-body comments for non-obvious setup or assertions.
- Keep tests human-friendly, jargon-free, and around 80-100 characters per line.
- For Rust, prefer `rstest` with `#[case::name(...)]` and aligned labeled parameters/comments.

## Code Placement
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

## Documentation
- Scope:
  - In changed scope, document caller-facing public APIs unless the target is a binary-only entrypoint.
  - If a change materially alters a module/file boundary, refresh module/file docs.
- Required docs:
  - Non-trivial public APIs: purpose, params, returns, notable failure behavior.
  - Trivial public APIs: brief purpose.
  - New or materially changed modules/files: top-level docs with purpose and caller-facing context.
  - If the language lacks native module docs, use the nearest file-level doc block/comment.
  - Add focused headings when useful: `Public API`, `Usage`, `Errors`, `Validation`, `Identifier Format`, `Precedence`.
  - `Public API` lists caller-facing entrypoints/types by role.
  - Rust external symbol mentions use [`TypeName`] plus trailing reference links when needed.
  - Never use `ignore` fences.
- Style:
  - Lead with a one-sentence purpose in plain language.
  - Prefer goal-oriented phrasing ("What you can do with this") over implementation terms.
  - Avoid jargon: no "materialization", "JIT", "framework-agnostic", "deterministic resolution", etc. Apply this to both code and documentation.
  - Keep examples practical and minimal.
  - Dense but accessible: full information without sacrificing readability.
- Review bar:
  - Missing required docs is blocking.
  - Docs must not contradict implementation.
  - Keep docs dense, not skeletal.
  - Do not backfill untouched legacy files solely for docs.
  - Add inline comments for non-obvious logic.

## Orchestration Plan
- Map each requirement to implementation step(s).
- Map each requirement to test step(s) or assertion(s).
- Include `## Requirement Trace Matrix` with requirement, implementation step ref(s), test step ref(s), and acceptance criteria.
- Keep `## External Symbols` current.
- In `## Implementation Steps`, each step includes `Action`, `Anchor`, `Lines` (approx), and `Order` (if needed).

## Orchestration Revisions
- Use these rules only when revising an existing plan or handling reviewer issue ledgers in the orchestrator workflow.
- Preserve issue IDs across revisions when root cause is unchanged.
- Include `acceptance_criteria` for each open issue ID (short, testable closure condition).
- Point to changed implementation/test sections that close each issue.
- Include `## Revision Impact Table` on revisions (changed hunk/step -> affected requirement(s) -> affected test(s)).
- Do not reopen resolved issues without new evidence.
