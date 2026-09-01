### Error-section ownership
Own all `# Errors`, `@throws`, or language-equivalent error docs for
public error-returning APIs in changed scope: existence, placement,
format, specificity, completeness. Block missing sections and vague
`may fail` coverage.

### Specific error triggers
Each error bullet names the condition that produces it; vague catch-all wording is insufficient.
Example: `Returns ParseError when the config file contains invalid TOML.`

### Error completeness
Error sections enumerate every reachable error variant/type/path the changed API can produce; list variants in enum/union order when one exists.

### Error doc fidelity
Error docs must not contradict implementation; each listed variant/type must be returnable from the function.

### Error doc format
Use the language's documentation convention and link syntax: Rust `# Errors` with `[`Error::Variant`]`; TypeScript `@throws` or equivalent project convention.

Prefer short in-text doc links plus reference definitions over long inline link targets.
Prefer `[Name]` in text plus one reference definition over repeated long inline targets.

### No vague error wording
Block vague triggers such as `when the operation fails`, `on error`, `if something goes wrong`, or `if invalid`.

### No error-doc legacy backfill
Do not backfill untouched legacy files solely for error docs.
