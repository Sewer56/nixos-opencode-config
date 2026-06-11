### Minimum visibility
Apply only to declarations introduced by the reviewed change or existing declarations whose visibility/export changed. Pre-existing unchanged declarations are out of scope.

For each changed visible declaration, gather before deciding:
- Name, file, current visibility, defining scope, and change source.
- Targeted repo search for direct uses, imports, re-exports, and barrel exports.
- Ignore self references, same-file references for cross-file demand, test-only references unless production visibility is required, and generated/vendor/build outputs unless changed.
- Override evidence: public re-export, documented API contract, derive/macro visibility requirement, trait-impl requirement, binary/example/FFI use, or reflection/string/DI use.

Evaluate top-to-bottom; first match wins.

| # | Condition | Decision |
|---|-----------|----------|
| 1 | Declaration is pre-existing and its visibility/export is unchanged | **NOT IN SCOPE** |
| 2 | Visibility exists only for tests, fixtures, examples under test, or `cfg(test)`/test-only modules | **NOT BLOCKING** |
| 3 | Re-exported by another module's public API, documented as an API contract, required by derive/macro expansion, required by a trait implementation, used by binary/example/FFI entry points, or accessed through reflection/string/DI wiring | **KEEP CURRENT** |
| 4 | Already at the narrowest visibility allowed by the language for observed non-test usage | **KEEP CURRENT** |
| 5 | Public/exported/API-visible declaration has zero non-test usage outside its defining module or package | **BLOCKING: reduce to private or the narrowest language visibility** |
| 6 | `pub(crate)`/`internal`/package-visible declaration has zero non-test callers outside its defining file | **BLOCKING: reduce to private/file-local visibility** |
| 7 | `pub(super)`/`protected`/parent-visible declaration has no non-test caller in the parent or protected scope | **BLOCKING: reduce to private or the narrowest scope with observed usage** |
| 8 | Barrel export or public re-export added by the change is not re-exported or consumed by any parent/public surface | **BLOCKING: remove the barrel export or narrow it** |

Minimum-visibility findings are `VISIBILITY` findings. If a new or visibility-modified declaration is more visible than actual usage requires, return a `BLOCKING` finding with the smallest narrowing diff.
