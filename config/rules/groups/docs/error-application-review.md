## Error Application Review

Apply style groups only to local error-doc text.
Keep error fidelity and specificity checks active.

{{ file="./rules/cards/docs/error-documentation.md" }}

Applied source docs must match current traced error facts and implementation.
Check functions, paths, lines, variants, and triggers.

Block dropped proposed variants or changed triggers.
Allow only if code proves the proposal obsolete.

With no traced error paths, use the language rule file's zero-path fallback.
This applies to proposed docs.
Block `TODO`, `TBD`, `FIXME`, `...`, and vague stubs in error docs.
Check both proposed and applied docs.

For findings with multiple diff blocks, label each block separately.
Put its own `**Lines: ~start-end**` before its diff fence.

{{ file="./rules/groups/style/readability.md" }}

{{ file="./rules/groups/style/wording.md" }}
