### Required documentation coverage
Public APIs (`pub`, `pub(crate)`, `export`, `public`) need purpose and parameter docs. Non-trivial public APIs also need returns, failure behavior, and examples when helpful. Non-trivial private APIs need purpose plus non-obvious parameters, returns, side effects, or invariants.
Do not flag: trivial private APIs with obvious names and direct behavior.

### Module and boundary docs
New or changed modules need top-level purpose/usage docs when the language or repo supports them. If a change alters module/file boundaries, refresh boundary docs.

### Documentation placement
Package docs cover import/usage shape; in-code docs cover exported symbols. Update both only when both exist and are affected. If examples are requested, place them in in-code API docs when the API owns them.

### Documentation fidelity
Docs must not contradict implementation. When documented surfaces are moved, renamed, or replaced, preserve or update affected docs.
Block stale names, options, defaults, links, examples, or behavior.

### Describe current behavior only
Code comments and in-code docs state the current contract and behavior only.
Never narrate old, removed, or previous behavior or versions in code ("no longer", "previously", "was", "used to", "before", "now" describing a change).
Old behavior may be referenced only in human-facing backward-compatibility documentation (e.g. release notes or migration notes), only when a genuine compatibility concern exists for consumers, and only on public APIs with an expectation of backward compatibility.
Never reference old behavior in private code, internals, tests, or helpers.
If a doc comment would need to mention old behavior, treat that as a signal that the draft must confirm a real public-API backward-compatibility obligation with the user (see draft change) rather than narrating the change.

### Inline readability comments
Non-trivial function bodies need short inline comments at logical steps when names and control flow do not explain intent.
Skip: trivial assignments, getters, direct delegation, and code already explained by names.
Example: `// Normalize aliases before validation so deprecated names share one error path.`

### Documentation style
Lead with a one-sentence purpose in plain language. Prefer goal-oriented phrasing. Use language-native doc-link syntax for types/variants when supported. Prefer short in-text doc links plus reference definitions over long inline link targets. Always include language tags on fenced code blocks; never use bare `ignore` fences.
Prefer `[Name]` in text plus one reference definition over repeated long inline targets.

### No legacy docs backfill
Do not backfill untouched legacy files solely for docs.

### Implementation-plan docs specificity
In implementation plans, name the affected documentation surface, audience, and behavioral change. Generic `update docs` notes are insufficient; exact prose or patch hunks are not required.

### Self-contained committed content
Committed code, comments, tests, docs, and commit messages must stand alone: never cite plan- or cohort-only identifiers in any form (`AC-1`, `[AC-#]`, `P1`, `Cnn`, review-finding ids) — describe the behavior instead.
