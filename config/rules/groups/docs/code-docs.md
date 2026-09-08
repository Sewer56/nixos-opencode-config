## Code Documentation

### Coverage
Choose detail for reader purpose, use, and consequential caveats.
Preserve required public API contracts and complete changed-API error docs.

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

Keep API-user facts, not behavior inventories or inferable internals.
Open with a plain one-line purpose summary.
Caveats: trailing `# Remarks` or equivalent.

Use native doc links and `#` sections for multiple aspects.
Name concrete mechanisms, not vague effects.

### Lists
- Summarize categories unless readers need their members.
- Use bullets for required lists, with exact code-font identifiers.
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
