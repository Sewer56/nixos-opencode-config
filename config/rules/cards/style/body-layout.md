### Body layout
Non-trivial bodies (tests included) need scannable shape: blank lines
between logical groups, purpose comments where non-obvious, a summary
on every test. Governs created or substantively rewritten bodies;
incidental edits need no re-layout.

Inline-comment wording and skip list: code-documentation card. Line
wrapping: formatters.

### Blank-line grouping
- One blank line between logical groups, never two; a group is one
  coherent step (validate, transform, decide, build result).
- Comments carry why; blank lines carry shape; both still separate.
- Tests: separate arrange, act, assert; split long arrange into
  harness, fixtures, inputs.
- Sub-group multi-step loop bodies like any other body.
- Skip single-group bodies; skipped tests still get their summary.

### Group-purpose comments
- Comment a group only when names and control flow leave intent
  non-obvious: one comment above, naming what it achieves; never every
  group. The inline-comment skip list applies.

### Test summaries
- Every test opens with a 1-2 line doc comment (`///` in Rust) above
  the attribute: condition plus pinned outcome. Name carries scenario,
  summary carries outcome.
- Interior comments never restate the summary; single-source applies.

### Ported and moved code
- Moved, ported, or rewritten regions get this layout even when the
  source was dense; carried-over density is not fidelity.

### Severity
- BLOCKING: a new or substantively rewritten body with 3+ logical groups and zero internal blank lines; a new or changed test missing its summary.
- ADVISORY: all other misses here: partial separation, missing group-purpose comment, arrange or loop sub-grouping, interior summary duplication.
