### Required documentation coverage
Public APIs (`pub`, `pub(crate)`, `export`, `public`) need purpose and
parameter docs; non-trivial ones also need returns, failure behavior, and
examples when helpful.

Non-trivial private APIs need purpose plus non-obvious parameters, returns,
side effects, or invariants. Do not flag trivial private APIs.

### Module and boundary docs
New or changed modules get top-level purpose/usage docs when the
language or repo supports them; refresh boundary docs when a change
alters module/file boundaries.

### Documentation placement
Package docs cover import/usage shape; in-code docs cover exported
symbols; update both only when both exist and are affected. Requested
examples go in in-code API docs when the API owns them.

### Examples
One concept per example, named for that concept; spin-offs get their own
example with a cross-reference.

Examples exercise real APIs on hermetic fixtures and show value static
configuration cannot express; never toy stand-ins (`[hook observed]`).

### Documentation fidelity
Docs must not contradict implementation; update docs for moved, renamed,
or replaced surfaces. Block stale names, options, defaults, links,
examples, or behavior.

### Single-source facts
State each fact once, on the surface that owns it.

No summary sentence plus a section repeating it.

Cross-reference another type's contract instead of restating it.

### Documentation scope
Document only key facts an API user needs; no fluff or feature notes.
Spin-offs get their own `#` section with a cross-reference. Edge cases are
one general sentence (`Empty chains are skipped.`), never enumerations.

### Describe current behavior only
- In-code docs state the current contract only, never old or removed
  behavior ("no longer", "previously", "was", "used to", "before", "now").
- Old behavior appears only in backward-compatibility docs (release or
  migration notes), only for genuine public-API compatibility concerns.
- Never in private code, internals, tests, or helpers.
- Needing old-behavior wording is a signal to confirm a public-API
  compatibility obligation with the user, not to narrate.

### Inline readability comments
Non-trivial bodies get short comments at logical steps when names and
control flow do not explain intent; skip trivial assignments, getters,
delegation, or names-explained code.

Example: `// Normalize aliases before validation so deprecated names
share one error path.`

### Documentation style
Lead with one plain-language purpose sentence; prefer goal-oriented
phrasing. Summary is one line; caveats go in a trailing `# Remarks`
section (or equivalent), never the summary.

Use language-native doc-link syntax; prefer `[Name]` plus reference
definitions over long or repeated inline targets.

Always tag fenced code blocks; never bare `ignore`. Split multi-aspect
docs into `#` sections, not dense paragraphs.

Name the concrete mechanism (`suppress it by returning `None``) over
the vague effect (`may suppress the event`).

### Lists over prose
- Inputs, outputs, parameters, variants, field mappings, branch points,
  and named-item sets are bullet lists, never comma-spliced prose.
- `Inputs:` and `Outputs:` label lines carry short noun-fragment
  bullets, one fact each, no periods.
- Branches and variants use `Label: sentence.` bullets ending in
  periods; field mappings use one sentence per bullet, source field in
  code font first; command, type, and parameter sets use one bullet per
  item: name in code font, dash, terse description.
- Lead-ins end with a colon and never restate bullets; delete the
  enumerated clause from the prose.
- Short sentence sets with nothing enumerable and single coherent
  mechanics stay prose.

### No legacy docs backfill
Do not backfill untouched legacy files solely for docs.

### Implementation-plan docs specificity
Plans name the docs surface, audience, and behavioral change; generic
`update docs` is insufficient, exact prose or hunks are not required.
