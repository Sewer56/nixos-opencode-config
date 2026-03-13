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
  - Avoid jargon: no "materialization", "JIT", "framework-agnostic", "deterministic resolution", etc.
  - Keep examples practical and minimal.
  - Dense but accessible: full information without sacrificing readability.
- Severity: Missing required docs is blocking. Docs must not contradict implementation.
- Limits: Keep docs dense, not skeletal. Do not backfill untouched legacy files solely for docs. Add inline comments only for non-obvious logic.
