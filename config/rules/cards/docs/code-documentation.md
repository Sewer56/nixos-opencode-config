### Required documentation coverage
Public APIs (`pub`, `pub(crate)`, `export`, `public`) need purpose and parameter docs. Non-trivial public APIs also need returns, failure behavior, and examples when helpful. Non-trivial private APIs need purpose plus non-obvious parameters, returns, side effects, or invariants.
Do not flag: trivial private APIs with obvious names and direct behavior.

### Module and boundary docs
New or changed modules need top-level purpose/usage docs when the language or repo supports them. If a change alters module/file boundaries, refresh boundary docs.

### Documentation placement
Package docs cover import/usage shape; in-code docs cover exported symbols. Update both only when both exist and are affected. If examples are requested, place them in in-code API docs when the API owns them.

### Documentation fidelity
Docs must not contradict implementation. When documented surfaces are moved, renamed, or replaced, preserve or update affected docs.
Bad: docs use an old option name after code renames it.
Good: docs and code use the same option name and behavior.

### Inline readability comments
Non-trivial function bodies need short inline comments at logical steps when names and control flow do not explain intent.
Skip: trivial assignments, getters, direct delegation, and code already explained by names.
Bad: a multi-step parser normalizes aliases before validation with no comment explaining why order matters.
Good: `// Normalize aliases before validation so deprecated names share one error path.`

### Documentation style
Lead with a one-sentence purpose in plain language. Prefer goal-oriented phrasing. Use language-native doc-link syntax for types/variants when supported. Always include language tags on fenced code blocks; never use bare `ignore` fences.

### No legacy docs backfill
Do not backfill untouched legacy files solely for docs.

### Machine-plan docs specificity
In machine plans, docs must appear in the relevant snippet or diff; generic `update docs` notes are insufficient.
