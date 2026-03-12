# Orchestrator Planning Rules

Use these rules for both planning and plan review.

## Plan Content
- No placeholders (`...`, `TODO`, comment-only test bodies).
- No undefined helpers/types/symbols in snippets.
- Map each requirement to implementation step(s).
- Map each requirement to test assertion(s).
- Include `## Requirement Trace Matrix` with requirement, implementation step ref(s), test step ref(s), and acceptance criteria.
- Keep `## External Symbols` current.
- In `## Implementation Steps`, each step includes `Action`, `Anchor`, `Lines` (approx), and `Order` (if needed).
- Insertions use normal code blocks with `Insert at: <anchor> (~start-end)`.
- Edits/removals use `diff` blocks; deletions include `Remove lines: ~start-end`.
- Import changes use a dedicated import `diff` block.
- If layout changes, include target tree and migration order.

## Revision Quality
- Preserve issue IDs across revisions when root cause is unchanged.
- Include `acceptance_criteria` for each open issue ID (short, testable closure condition).
- Point to changed implementation/test sections that close each issue.
- Include `## Revision Impact Table` on revisions (changed hunk/step -> affected requirement(s) -> affected test(s)).
- Do not reopen resolved issues without new evidence.

## Design Discipline
- Keep changes minimal.
- Reuse existing patterns.
- Split catch-all files into focused modules.
- Keep top-level orchestration in parent module/file entrypoint.
- Keep data-holder models in dedicated model modules.
- Keep non-public helper types local.
- Keep conversions with related type definitions.
- Co-locate tests with the module they validate.
- Keep visibility minimal.
- Within files, order declarations from most public to most private.
- Avoid broad refactors unless required or requested.
- Avoid dead code, debug/dev-only logging, and unnecessary abstractions.
- Avoid duplicate tests and test helpers.

## Documentation
- Scope: In changed scope, document public APIs unless the target is a binary-only entrypoint.
- Required docs:
  - Non-trivial public APIs: description, parameters, and returns.
  - Trivial public APIs: brief description only.
  - New or materially changed modules/files: top-level docs (e.g. module docs) covering purpose and key context when supported.
- Severity:
  - Missing required docs and trivial API blurb is blocking.
  - Docs must not contradict implementation.
- Limits:
  - Keep doc changes concise when behavior changes.
  - Do not backfill untouched legacy files solely for docs.
  - Add inline comments only for non-obvious logic.
