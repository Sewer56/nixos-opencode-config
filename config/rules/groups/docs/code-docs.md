## Code Documentation

### Coverage and placement
Non-trivial private APIs need purpose and non-obvious contract details.
Do not flag trivial private APIs.

Refresh boundary docs when module/file boundaries change.

Package docs cover import/usage shape.
In-code docs cover exported symbols.
Update both only when both exist and change.

Put requested examples in in-code API docs when the API owns them.
Never backfill untouched legacy files solely for docs.

### Examples and style
Use one concept per example, named for that concept.
Spin-offs get their own example and `#` section with cross-references.

Examples use real APIs with hermetic fixtures.
Show what static configuration cannot express.

Block docs that misrepresent implementation or contain stale references.

State facts once on their owning surface.
Cross-reference other contracts.

Document only key API-user facts, not feature notes.
Summarize edge cases in one general sentence, not enumerations.

Open with one plain-language purpose sentence, preferably goal-oriented.
Keep the summary on one line.
Put caveats in trailing `# Remarks` or equivalent.

Use language-native doc links and `#` sections for multi-aspect docs.
Name concrete mechanisms, not vague effects.

### Lists over prose
- Bullet each input, output, parameter, variant, mapping, branch or set item.
- `Inputs:` and `Outputs:` bullets: short noun fragments, one fact, no periods.
- Branches/variants use `Label: sentence.` bullets.
- Field mappings use one sentence per bullet, source field in code font first.
- Command/type/parameter bullets: code-font name, dash, terse description.
- Lead-ins never restate bullets.
- Keep short non-enumerable sentences and single coherent mechanics in prose.

{{ file="./rules/cards/style/adhd-format.md" }}

### Body layout
Comment steps in non-trivial bodies if names/control flow leave intent unclear.

#### Blank-line grouping
Apply to new or substantively rewritten non-trivial bodies, including tests.
Incidental edits need no re-layout.
Leave line wrapping to formatters.

- Separate coherent steps with one blank line.
- Comments explain why without replacing blank lines.
- Tests separate arrange, act, assert.
- Split long arrange into harness, fixtures, inputs.
- Sub-group multi-step loop bodies.
- Skip single-group bodies.
- Place each required group-purpose comment once above its group.
- Apply layout to moved, ported, or rewritten regions despite source density.

### Severity
- BLOCKING: 3+ groups with zero internal blank lines.
- All other issues are ADVISORY.
