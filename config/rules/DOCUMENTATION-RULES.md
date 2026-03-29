# Documentation Rules

## Scope
- In changed scope, document caller-facing public APIs unless the target is a binary-only entrypoint.
- If a change materially alters a module/file boundary, refresh module/file docs.

## Required Docs
- Non-trivial public APIs: purpose, params, returns, notable failure behavior.
- Trivial public APIs: brief purpose.
- New or materially changed modules/files: top-level docs with purpose and caller-facing context.
- If the language lacks native module docs, use the nearest file-level doc block/comment.
- Add focused headings when useful: `Public API`, `Usage`, `Errors`, `Validation`, `Identifier Format`, `Precedence`.
- `Public API` lists caller-facing entrypoints/types by role.
- Rust external symbol mentions use [`TypeName`] plus trailing reference links when needed.
- Never use `ignore` fences.

## Style
- Lead with a one-sentence purpose in plain language.
- Prefer goal-oriented phrasing ("What you can do with this") over implementation terms.
- Avoid jargon: no "materialization", "JIT", "framework-agnostic", "deterministic resolution", etc. Apply this to both code and documentation.
- Keep examples practical and minimal.
- Dense but accessible: full information without sacrificing readability.

## Review Bar
- Missing required docs is blocking.
- Docs must not contradict implementation.
- Keep docs dense, not skeletal.
- Do not backfill untouched legacy files solely for docs.
- Add inline comments for non-obvious logic.
