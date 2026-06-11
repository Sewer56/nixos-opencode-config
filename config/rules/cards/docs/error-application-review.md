### Error application fidelity
Applied source docs must match the proposed/cached error section: functions, paths, lines, variants, and triggers align.
Block dropped proposed variants or trigger changes unless code evidence proves the proposal obsolete.

### Zero-path fallback
When no error paths were traced, proposed docs must apply the language rule file's zero-path fallback.

### No error placeholders
Block `TODO`, `TBD`, `FIXME`, `...`, and vague stubs in proposed or applied error docs.

### Per-hunk review diffs
When a finding contains multiple diff blocks, label each block with its own `**Lines: ~start-end**` before the diff fence.
