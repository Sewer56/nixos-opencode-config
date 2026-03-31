# Documentation Rules

## Scope
- In changed scope, document public APIs and exports. In Rust, treat `pub` and `pub(crate)` items as public for this rule.
- In changed scope, document non-trivial private APIs.
- For library or package APIs, update both package-level docs (`README.md` or the nearest usage doc) and in-code API docs when both surfaces exist in the changed scope.
- If a change materially alters a module/file boundary, refresh module/file docs.
- Update existing documentation as needed.
- Do not remove existing documentation unless it is incorrect or no longer applies.
- When moving or renaming documented public APIs, exports, non-trivial private APIs, or modules/files, preserve or replace the affected docs explicitly.

## Required Docs
- Public APIs and exports: brief purpose. Document parameters on public APIs.
- Non-trivial public APIs and exports: also document returns, notable failure behavior, and examples when requested or materially helpful.
- Non-trivial private APIs: brief purpose plus any non-obvious parameters, returns, side effects, or invariants.
- Trivial private APIs do not need full API docs.
- If the user explicitly asks for examples, add a practical example to the relevant public API docs, not only to package-level docs.
- New or materially changed modules/files: top-level docs with purpose and usage context.
- Package-level docs cover import and usage shape; in-code API docs cover the exported symbols themselves.
- If the language lacks native module docs, use the nearest file-level doc block/comment.
- Add focused headings when useful: `Public API`, `Arguments`, `Returns`, `Examples`, `Usage`, `Errors`, `Validation`, `Identifier Format`, `Precedence`.
- `Public API` lists public entrypoints/types by role.
- Rust external symbol mentions use [`TypeName`] plus trailing reference links when needed.
- Never use `ignore` fences.

## Style
- Lead with a one-sentence purpose in plain language.
- For sectioned function and method docs, use this order: short summary, `Arguments`, `Returns`, then `Examples`.
- Prefer goal-oriented phrasing ("What you can do with this") over implementation terms.
- Avoid jargon: no "materialization", "JIT", "framework-agnostic", "deterministic resolution", etc. Apply this to both code and documentation.
- Keep examples practical and minimal.
- When the doc format supports fenced examples, include a language tag such as `rust`.
- Dense but accessible: full information without sacrificing readability.

## Review Bar
- Missing required docs is blocking.
- Docs must not contradict implementation.
- Keep docs dense, not skeletal.
- When package-level docs and in-code API docs are both in scope, missing either side is blocking.
- If examples were explicitly requested, README-only examples are insufficient.
- Missing docs for non-trivial private APIs in changed scope is blocking.
- In machine plans, required docs must appear concretely in the relevant implementation step snippet/diff; a generic `update docs` note is insufficient.
- Do not backfill untouched legacy files solely for docs.
- Add inline comments for non-obvious logic.
