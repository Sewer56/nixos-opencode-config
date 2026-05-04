## Error Documentation Rules

### Required Error Docs
- Public error-returning APIs (`pub`, `pub(crate)`, `export`, `public`): `# Errors` section with one bullet per variant/type and a specific trigger condition. Private/internal functions do not need `# Errors` docs. Required format:
  ```
  /// # Errors
  /// - Returns [`Error::Variant`] when <specific condition>.
  /// - Returns [`Error::OtherVariant`] when <specific condition>.
  ```
  Each bullet must let a reader predict exactly which input/state produces which variant. Never write vague triggers:
  - ~~"when the operation fails"~~
  - ~~"on error"~~
  - ~~"if something goes wrong"~~

### Error Doc Style
- Reference error variants using the language's doc-link syntax (Rust: `[`Error::Variant`]`; TypeScript: backtick-quoted `ErrorVariant`).
- Write trigger conditions as concrete input/state predicates: "when `paths` contains entries not valid relative to the installer root" not "when validation fails".
- Within `# Errors`, list variants in the same order as the enum/union definition when one exists.

### Error Review Blocking Criteria
- `# Errors` sections must enumerate every reachable error variant with a specific trigger condition.
- Vague bullets (e.g. "when the operation fails") block the review.
- Error documentation must not contradict implementation — each listed variant must be actually returnable from the function.
- Do not backfill untouched legacy files solely for error docs.
