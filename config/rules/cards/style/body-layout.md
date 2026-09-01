### Body layout
Non-trivial bodies (tests included) need scannable shape: blank lines
between logical groups, purpose comments where non-obvious, a summary
on every test. Governs created or substantively rewritten bodies;
incidental edits need no re-layout.

Inline-comment wording and skip list: code-documentation card. Line
wrapping: formatters.

### Blank-line grouping
- One blank line between logical groups, never two. A group is one coherent step: validate, transform, decide, build result.
- Comments carry why; blank lines carry shape. Comment-marked steps still need separation.
- Tests: separate arrange, act, assert. Split long arrange blocks into harness, fixtures, inputs.
- Loops: sub-group a multi-step per-iteration body like any other body.
- Skip: bodies with a single logical group, including short single-group tests. Skipped tests still get their summary.

### Group-purpose comments
- Comment a group only when names and control flow leave its intent non-obvious: one comment above the group naming what it achieves. Never comment every group.
- Apply the inline-comment skip list owned by the code-documentation card.

### Test summaries
- Every test opens with a 1-2 line doc comment (`///` in Rust) above the test attribute: the condition under test plus the outcome it pins. The name carries the scenario; the summary carries the outcome.
- Interior comments must not restate the summary; single-source rules apply.

### Ported and moved code
- Regions moved, ported, or rewritten in a change get this layout even when the source was dense. Carried-over density is not fidelity.

### Severity
- BLOCKING: a new or substantively rewritten body with 3+ logical groups and zero internal blank lines; a new or changed test missing its summary.
- ADVISORY: all other misses here: partial separation, missing group-purpose comment, arrange or loop sub-grouping, interior summary duplication.
