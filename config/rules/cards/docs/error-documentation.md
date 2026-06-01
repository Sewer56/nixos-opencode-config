### Error-section ownership
Own all `# Errors`, `@throws`, or language-equivalent error documentation for public error-returning APIs in changed scope: existence, placement, format, specificity, and completeness.
Bad: public error-returning API has no error section or only says `may fail`.
Good: error docs list each variant/type with a concrete trigger.

### Specific error triggers
Each error bullet must name the condition that produces it. Vague catch-all wording is insufficient.
Bad: `Returns Error if something goes wrong.`
Good: `Returns ParseError when the config file contains invalid TOML.`

### Error completeness
Error sections must enumerate every reachable error variant/type/path that the changed API can produce. List variants in enum/union order when one exists.

### Error doc fidelity
Error documentation must not contradict implementation. Each listed variant/type must be actually returnable from the function.

### Error doc format
Use the language's documentation convention and link syntax: Rust `# Errors` with `[`Error::Variant`]`; TypeScript `@throws` or equivalent project convention. Prefer short in-text doc links plus reference definitions over long inline link targets.
Bad: `[Name](long.target.path)` repeated inline. Good: `[Name]` in text and `[Name]: long.target.path` after the section.

### No vague error wording
Block vague triggers such as `when the operation fails`, `on error`, `if something goes wrong`, or `if invalid`.

### No error-doc legacy backfill
Do not backfill untouched legacy files solely for error docs.
