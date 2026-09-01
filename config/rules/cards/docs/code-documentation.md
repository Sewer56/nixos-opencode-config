### Required documentation coverage
Public APIs (`pub`, `pub(crate)`, `export`, `public`) need purpose and
parameter docs; non-trivial ones also need returns, failure behavior, and
examples when helpful.

Non-trivial private APIs need purpose plus non-obvious parameters, returns,
side effects, or invariants. Do not flag trivial private APIs.

### Module and boundary docs
New or changed modules need top-level purpose/usage docs when the language or repo supports them.

If a change alters module/file boundaries, refresh boundary docs.

### Documentation placement
Package docs cover import/usage shape; in-code docs cover exported symbols.

Update both only when both exist and are affected.

If examples are requested, place them in in-code API docs when the API owns them.

### Examples
One concept per example, named for that concept.

Spin-off behavior gets its own example with a cross-reference.

Examples exercise real APIs on hermetic fixtures and show value static configuration cannot express; never toy stand-ins (`[hook observed]`).

### Documentation fidelity
Docs must not contradict implementation.

When documented surfaces are moved, renamed, or replaced, preserve or update affected docs.

Block stale names, options, defaults, links, examples, or behavior.

### Single-source facts
State each fact once, on the surface that owns it.

No summary sentence plus a section repeating it.

Cross-reference another type's contract instead of restating it.

### Documentation scope
Document only the key facts an API user needs; no fluff, no exhaustive feature notes.

Spin-off or related behavior gets its own `#` section with a cross-reference, not more paragraphs.

State edge cases as one general sentence (`Empty chains are skipped.`); never enumerate permutations.

### Describe current behavior only
- **Current contract only**: Code comments and in-code docs state the current contract and behavior only.
- **Never narrate old behavior**: Never narrate old, removed, or previous behavior or versions in code ("no longer", "previously", "was", "used to", "before", "now" describing a change).
- **Backward-compatibility docs only**: Old behavior may be referenced only in human-facing backward-compatibility documentation (e.g. release notes or migration notes).
  - Reference old behavior only when a genuine compatibility concern exists for consumers.
  - Reference old behavior only on public APIs with an expectation of backward compatibility.
- **Never in private code**: Never reference old behavior in private code, internals, tests, or helpers.
- **Signal to confirm, not narrate**: If a doc comment would need to mention old behavior, treat that as a signal rather than narrating the change.
  - The signal means the draft must confirm a real public-API backward-compatibility obligation with the user (see draft change).

### Inline readability comments
Non-trivial function bodies need short inline comments at logical steps when names and control flow do not explain intent.

Skip: trivial assignments, getters, direct delegation, and code already explained by names.

Example: `// Normalize aliases before validation so deprecated names share one error path.`

### Documentation style
Lead with a one-sentence purpose in plain language.

Prefer goal-oriented phrasing.

Use language-native doc-link syntax for types/variants when supported.

Prefer short in-text doc links plus reference definitions over long inline link targets.

Always include language tags on fenced code blocks; never use bare `ignore` fences.

Prefer `[Name]` in text plus one reference definition over repeated long inline targets.

Summary is one line.

Scope limits and caveats go in a trailing `# Remarks` section (or language equivalent), never in the summary paragraph.

Split multi-aspect docs into `#` sections, not dense paragraphs.

Name the concrete mechanism (`suppress it by returning `None``) over the vague effect (`may suppress the event`).

### Lists over prose
- **Enumerations are lists**: Inputs, outputs, parameters, variants, field mappings, branch points, and named-item sets are bullet lists, never comma-spliced prose.
- **Label data groups**: `Inputs:` and `Outputs:` label lines carry short noun-fragment bullets, one fact each, no periods.
- **Label named aspects**: Branches and variants use `Label: sentence.` bullets ending in periods.
  - Field mappings use one sentence per bullet, source field in code font first.
  - Command, type, and parameter sets use one bullet per item: name in code font, dash, terse description.
- **Lead-in then bullets**: Lead-ins end with a colon and never restate the bullets; delete the enumerated clause from the prose.
- **Narrative stays prose**: Short sentence sets with nothing enumerable and single coherent mechanics stay prose.

### No legacy docs backfill
Do not backfill untouched legacy files solely for docs.

### Implementation-plan docs specificity
In implementation plans, name the affected documentation surface, audience, and behavioral change.

Generic `update docs` notes are insufficient; exact prose or patch hunks are not required.
