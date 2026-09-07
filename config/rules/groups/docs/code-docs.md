## Code Documentation

### Coverage
Private APIs need purpose and non-obvious contracts unless trivial.
Refresh changed module/file boundary docs.

Package docs: import/usage; code docs: exports.
Update both only when both exist and change.
Put requested API-owned examples in in-code docs.
No docs-only backfill of untouched legacy.

### Examples and style
Name each example for its one concept.
Give spin-offs own examples and `#` sections with cross-links.
Examples use real APIs with hermetic fixtures.
Examples show what static configuration cannot express.

Block false claims and stale references.
State facts once where owned.
Link other contracts.
Keep key API-user facts, not feature notes.

Summarize edge cases in one general sentence.
Open with a plain purpose sentence, ideally goal-oriented.
Use one-line summaries.
Caveats: trailing `# Remarks` or equivalent.

Use native doc links and `#` sections for multiple aspects.
Name concrete mechanisms, not vague effects.

### Lists
- Bullet each input/output/parameter/variant/mapping/branch/set item.
- `Inputs:`/`Outputs:`: short noun fragments, one fact, no periods.
- Branches/variants: `Label: sentence.`
- Mappings: one sentence per bullet, code-font source field first.
- Commands/types/params: code-font name, dash, terse text.
- Lead-ins never restate bullets.
- Keep short non-list sentences and single coherent mechanics in prose.

{{ file="./rules/cards/style/adhd-format.md" }}

### Body layout
Comment non-trivial bodies' steps if names/flow obscure intent.

Group new/substantively rewritten non-trivial bodies, including tests.
No re-layout for incidental edits.
Formatters own line wrapping.
- Separate coherent steps with one blank line.
- Comments explain why, not replace blank lines.
- Tests separate arrange, act, assert.
- Split long arrange into harness, fixtures, inputs.
- Sub-group multi-step loop bodies.
- Skip single-group bodies.
- Put required purpose comments once above their group.
- Group moved/ported/rewritten regions even if dense.

### Severity
- BLOCKING: 3+ groups with zero internal blank lines.
- All other issues: ADVISORY.
