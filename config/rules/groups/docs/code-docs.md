## Code Documentation

### Coverage and placement
Public APIs (`pub`, `pub(crate)`, `export`, `public`) need purpose and parameter docs.
Non-trivial public APIs also need returns, failure behavior, and helpful examples.

Non-trivial private APIs need purpose and non-obvious parameters, returns, side effects, or invariants.
Do not flag trivial private APIs.

New/changed modules get top-level purpose/usage docs where supported by language or repo.
Refresh boundary docs when module/file boundaries change.

Package docs cover import/usage shape; in-code docs cover exported symbols.
Update both only when both exist and change.

Put requested examples in in-code API docs when the API owns them.
Never backfill untouched legacy files solely for docs.

### Examples and facts
Use one concept per example, named for that concept.
Spin-offs get their own example and `#` section with cross-references.

Examples exercise real APIs on hermetic fixtures and show value static configuration cannot express.
Never use toy stand-ins (`[hook observed]`).

Docs must match implementation, including moved, renamed, or replaced surfaces.
Block stale names, options, defaults, links, examples, or behavior.

State each fact once on its owning surface, not in both summary and section.
Cross-reference another type's contract instead of restating it.

Document only key API-user facts, without fluff or feature notes.
Describe edge cases in one general sentence, never enumerations.

### Inline readability comments
Comment logical steps in non-trivial bodies when names/control flow do not explain intent.
Skip trivial assignments, getters, delegation, and names-explained code.

### Documentation style
Lead with one plain-language purpose sentence; prefer goal-oriented phrasing.
Keep the summary on one line; put caveats in a trailing `# Remarks` section or equivalent.

Use language-native doc links and `#` sections for multi-aspect docs.
Name the concrete mechanism (`suppress it by returning None`), not the vague effect (`may suppress the event`).

### Lists over prose
- Use bullets for inputs, outputs, parameters, variants, field mappings, branch points, and named-item sets.
- `Inputs:` and `Outputs:` labels take short noun-fragment bullets: one fact each, no periods.
- Branches/variants use `Label: sentence.` bullets ending in periods.
- Field mappings use one sentence per bullet, source field in code font first.
- Command, type, and parameter sets use one bullet per item: code-font name, dash, terse description.
- Lead-ins never restate bullets.
- Keep short non-enumerable sentence sets and single coherent mechanics in prose.

{{ file="./rules/cards/style/adhd-format.md" }}

### Body layout
Apply to new or substantively rewritten non-trivial bodies, including tests.
Incidental edits need no re-layout.
Use Inline readability comments above for comment wording and skips.
Leave line wrapping to formatters.

### Blank-line grouping
- Separate logical groups with one blank line: each group is one coherent step.
- Comments carry why; blank lines carry shape; both still separate.
- Tests separate arrange, act, assert; split long arrange into harness, fixtures, inputs.
- Sub-group multi-step loop bodies like any other body.
- Skip single-group bodies.
- Place each required group-purpose comment once above its group.
- Apply layout to moved, ported, or rewritten regions even if the source was dense.
- Carried-over density is not fidelity.

### Severity
- BLOCKING: 3+ groups with zero internal blank lines.
- ADVISORY: partial separation, no group-purpose comment, arrange/loop sub-grouping, summary duplication, and everything else.
