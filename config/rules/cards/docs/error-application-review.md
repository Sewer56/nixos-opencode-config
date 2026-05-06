### Error application fidelity
Applied source docs must match the intended or cached proposed error doc section. Function names, paths, line numbers, variants, and triggers must align.
Bad: source doc drops one proposed error variant.
Good: source doc preserves every proposed bullet with matching trigger.

### Zero-path fallback
When no error paths were traced, proposed docs must apply the language rule file's zero-path fallback.
Bad: omit docs because no error paths were traced.
Good: apply the language-specific zero-path wording.

### No error placeholders
Block `TODO`, `TBD`, `FIXME`, `...`, and vague stubs in proposed or applied error docs.
Bad: `TODO: document errors`.
Good: concrete error docs or explicit zero-path fallback.

### Per-hunk review diffs
When a finding contains multiple diff blocks, label each block with its own `**Lines: ~start-end**` before the diff fence.
Bad: one finding-level range for multiple hunks.
Good: each hunk has its own bold line label.
