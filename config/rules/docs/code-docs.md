## Code Documentation

### Documentation context

Adapt all documentation to audience, reader task and document type.

Match concise peers' structure and depth unless requirements justify departures.

Apply source/API duties only to source/API docs.
Preserve explicit requirements, contracts and complete changed-API errors.

Keep consequential safety/compatibility caveats.
Maintainer docs explain needed mechanisms.

Write for action, decisions or required understanding, not a fact inventory.

Before validation/handoff, separately prune scoped doc changes.
Delete unnecessary sections, examples and sentences before polishing.

Omit consequences clear from defaults, definitions or examples.
Truth or possible usefulness alone is insufficient.

State facts once where owned; link configuration/contracts instead of repeating.
Never relocate unnecessary detail into new docs.

Examples show choices/usage, not every field.
Keep necessary content; no length quotas.

### Coverage
Private APIs need purpose and non-obvious contracts unless trivial.
Refresh changed module/file boundary docs.

Package docs: import/usage; code docs: exports.
Update both only when both exist and change.
Put requested API-owned examples in in-code docs.
No docs-only backfill of untouched legacy.

### Examples and style
Name each example for its one concept.
Add examples, sections and cross-links only when they help readers.

Examples use real APIs with hermetic fixtures.
Examples show what static configuration cannot express.

Block false claims and stale references.

Open with a plain one-line purpose summary.
Caveats: trailing `# Remarks` or equivalent.

Use native doc links and `#` sections for multiple aspects.
Name concrete mechanisms when readers need them, not vague effects.

### Lists
- Summarize categories unless readers need their members.
- Use bullets for required lists, with exact code-font identifiers.
- Lead-ins never restate bullets.
- Keep single coherent mechanics in prose.

{{ file="./rules/write/adhd-format.md" }}

### Severity
Other documentation issues are ADVISORY.
