### Public API classification
Gather evidence per item for each override condition. Check all that apply:

- **Re-exports** (rule 1): grep for re-exports (`pub use`, `export.*from`, `__init__.py` re-imports, `public fun` delegating). Count only cross-module re-exports. Same-module = internal organization, not external demand.
- **Doc contract** (rule 2): read doc comments for API guarantees or `# API` sections.
- **Derive macros** (rule 3): check if item is field on struct/enum with derive attributes that access fields or generate public methods.
- **Trait impl** (rule 4): check if item appears in `impl Trait for Type` where type is not fully private.
- **Binary/FFI** (rule 5): grep binary targets, examples, FFI bindings for the item.
- **Reflection/DI** (rule 6): grep for string-name lookups (e.g. `getattr`); see language file for full per-runtime list.

Rules evaluated top-to-bottom; first match wins.

| # | Condition | Decision |
|---|-----------|----------|
| 1 | Re-exported by another module's public API | **KEEP PUBLIC** |
| 2 | Documented as public API contract (doc comments, `# API` sections) | **KEEP PUBLIC** |
| 3 | Required by derive macro on non-fully-private type — any derive accessing fields or generating public methods forces field visibility ≥ type visibility (see language file; e.g. `serde`, `pyo3` families) | **KEEP PUBLIC** |
| 4 | Part of trait impl on non-fully-private type (visibility must satisfy trait contract) | **KEEP PUBLIC** |
| 5 | Referenced in binary, example, or FFI binding outside module | **KEEP PUBLIC** |
| 6 | Accessed via reflection/string reference or DI wiring (invisible to grep; see language file; e.g. `getattr`) | **KEEP PUBLIC** |
| 7 | Visibility contains `doc(hidden)` — author flagged intentionally hidden | **MANUAL REVIEW** |
| 8 | `candidate-medium` AND used only in code-generated files (`Code generated`, `DO NOT EDIT` headers) | **CANDIDATE LOW** |
| 9 | `candidate-high` | **CANDIDATE HIGH** |
| 10 | `candidate-medium` | **CANDIDATE MEDIUM** |
| 11 | `review` | **MANUAL REVIEW** |

Decision table = sole authority for initial classification. Rules 1–6 → KEEP PUBLIC. Rule 7 → MANUAL REVIEW (doc-hidden). Rule 8 → demote MEDIUM to LOW. Restriction hint override: if hint is `none` AND table outcome ≠ KEEP PUBLIC → reclassify as MANUAL REVIEW (no specific visibility change can be recommended). KEEP PUBLIC items are correctly public regardless of hint.

### Restriction hint mapping
For every candidate, map Restriction Hint to target visibility in diff:

- `can-be-private` → remove visibility keyword entirely (Rust: drop `pub`/`pub(crate)`; TS: drop `export`; Python: `_` prefix or remove from `__all__`; Go: lowercase first letter; Java: `private`; Kotlin: `private`)
- `can-be-package-private` → remove `public`/`protected`, leaving default access (Java only)
- `can-be-internal` → add `internal` modifier (Kotlin only)
- `can-be-pub-super` → `pub(super)` (Rust only)
- `can-be-pub-in(<path>)` → `pub(in <path>)` using path from collector hint (Rust only)
- `none` → needs current scope but over-exposed vs external demand; restriction hint override above reclassifies as MANUAL REVIEW
