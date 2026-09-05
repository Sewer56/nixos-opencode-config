### Body layout
Governs new or substantively rewritten non-trivial bodies, including tests.
Incidental edits need no re-layout.

Inline-comment wording and skip list: code-documentation card.
Line wrapping: formatters.

### Blank-line grouping
- One blank line between logical groups, never two; a group is one
  coherent step (validate, transform, decide, build result).
- Comments carry why; blank lines carry shape; both still separate.
- Tests: separate arrange, act, assert; split long arrange into
  harness, fixtures, inputs.
- Sub-group multi-step loop bodies like any other body.
- Skip single-group bodies; skipped tests still get their summary.

### Group-purpose comments
- Place each required group-purpose comment once above its group.

### Test summaries
- Every test opens with a 1-2 line doc comment (`///` in Rust) above
  the attribute: condition plus pinned outcome. Name carries scenario,
  summary carries outcome.
- Interior comments never restate the summary; single-source applies.

### Ported and moved code
- Moved, ported, or rewritten regions get this layout even when the
  source was dense; carried-over density is not fidelity.

### Severity
- BLOCKING: 3+ groups with zero internal blank lines; test missing its
  summary.
- ADVISORY: everything else: partial separation, no group-purpose
  comment, arrange/loop sub-grouping, summary duplication.
