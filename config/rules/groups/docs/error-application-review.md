## Error Application Review

Apply style groups below only to local error-doc text; keep error fidelity/specificity checks active.

### Error documentation

{{ file="./rules/cards/docs/error-documentation.md" }}

### Application fidelity
Applied source docs must match current traced error facts and implementation: functions, paths, lines, variants, and triggers.

Block dropped proposed variants or trigger changes unless code evidence proves the proposal obsolete.

When no error paths were traced, proposed docs must apply the language rule file's zero-path fallback.
Block `TODO`, `TBD`, `FIXME`, `...`, and vague stubs in proposed or applied error docs.

For findings with multiple diff blocks, label each block with its own `**Lines: ~start-end**` before the diff fence.

{{ file="./rules/groups/style/readability.md" }}

{{ file="./rules/groups/style/wording.md" }}
