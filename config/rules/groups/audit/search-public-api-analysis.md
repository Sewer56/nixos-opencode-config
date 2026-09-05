## Public API Analysis

Gather only evidence needed by the override table:
- Re-exports: grep cross-module `pub use`, `export.*from`, `__init__.py` re-imports, and delegating `public fun`.
- Same-module re-exports are internal organization.
- Doc contract: read doc comments and `# API` sections for public guarantees.
- Derives: check fields on structs/enums with derives accessing fields or generating public methods.
- Trait impl: check `impl Trait for Type` when the type is not fully private.
- Binary/FFI: grep binaries, examples, and FFI bindings.
- Reflection/DI: grep string-name lookups such as `getattr`; use language rules for runtime-specific cases.

### Classification
Evaluate top-to-bottom; first match wins.
The ordered table is the sole initial-classification authority.

On non-fully-private types, derives accessing fields or generating public methods force field visibility ≥ type visibility.

Use the language file for derive families such as `serde` and `pyo3`, and reflection/DI cases invisible to grep.

| # | Condition | Decision |
|---|-----------|----------|
| 1 | Re-exported by another module's public API | **KEEP PUBLIC** |
| 2 | Documented public API contract (doc comments, `# API` sections) | **KEEP PUBLIC** |
| 3 | Required by derive on non-fully-private type under the rule above | **KEEP PUBLIC** |
| 4 | Trait impl on non-fully-private type requiring trait-contract visibility | **KEEP PUBLIC** |
| 5 | Referenced in binary, example, or FFI binding outside module | **KEEP PUBLIC** |
| 6 | Accessed via reflection/string reference or DI wiring | **KEEP PUBLIC** |
| 7 | Visibility contains `doc(hidden)`: intentionally hidden | **MANUAL REVIEW** |
| 8 | `candidate-medium`, used only in code-generated files (`Code generated`, `DO NOT EDIT` headers) | **CANDIDATE LOW** |
| 9 | `candidate-high` | **CANDIDATE HIGH** |
| 10 | `candidate-medium` | **CANDIDATE MEDIUM** |
| 11 | `review` | **MANUAL REVIEW** |

If Restriction Hint is `none` and outcome is not KEEP PUBLIC, override to MANUAL REVIEW: no specific visibility change is recommendable.
KEEP PUBLIC items are correctly public regardless of hint.

### Restriction hint mapping
Map each candidate's Restriction Hint to target visibility in its diff:
- `can-be-private`: remove visibility keyword entirely.
- Rust drops `pub`/`pub(crate)`; TS drops `export`; Python uses `_` prefix or removes from `__all__`.
- Go lowercases the first letter; Java/Kotlin use `private`.
- `can-be-package-private`: remove `public`/`protected`, leaving default access (Java only).
- `can-be-internal`: add `internal` (Kotlin only).
- `can-be-pub-super`: use `pub(super)` (Rust only).
- `can-be-pub-in([[path]])`: use `pub(in [[path]])` with the collector hint's path (Rust only).
- `none`: needs current scope but is over-exposed versus external demand; apply the MANUAL REVIEW override above.
