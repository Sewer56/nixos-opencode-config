---
mode: subagent
hidden: true
description: Enumerates public items in one module and cross-references usage across the repo
model: sewer-bifrost/synthetic/hf:zai-org/GLM-5.1
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  grep: allow
  glob: allow
  list: allow
  bash: allow
  todowrite: allow
  external_directory: allow
---

Enumerate all public items in one module and count how many times each is used outside that module.

# Inputs

- `target_path`: absolute path to the module root, crate directory, or source file
- `language`: `rust` | `typescript` | `python` | `go` | `java` | `kotlin`
- `repo_root`: absolute path to the repository root
- `specific_paths`: (optional) list of absolute paths to specific files/directories within `target_path`. If provided and the module has more than 80 public items, enumerate only items whose file is within one of these paths and note the truncation in the summary.

# Workflow

## 1. Enumerate

List every public item in `target_path` per the language file (described in `# Language-Specific Rules`).

Skip items inside `#[cfg(test)]` modules, `#[test]` functions, and test-only files (files matching the test patterns in the language file). These are only visible during test compilation and are not part of the production API surface.

For each item record: name, file path, line number, visibility keyword.

If `language` is `rust`: run `cargo public-api --simplified 2>/dev/null`. If exit code is 0 and output is non-empty, use the returned item signatures, then grep for each returned item to determine its file path and line number. If grep fails to locate an item, fall back to full grep-based enumeration. Regardless of cargo-public-api success, always grep for `pub(crate)`, `pub(super)`, and `pub(in` — cargo-public-api does not report scoped-pub items. For all other languages, use grep. Record which method was used (`cargo-public-api` or `grep`).

## 2. Cross-reference

For each item, search the entire repo for its name. If the module has more than 80 public items and `specific_paths` was provided, enumerate only items in `specific_paths` and note the total count was too large for full enumeration. Complete every item — do not skip or batch items. Classify every match:

- **External**: outside module boundary, not a test file
- **Test (external)**: outside module boundary, matches test patterns
- **Internal non-test**: under module boundary, not the defining file, not a test file
- **Internal test**: under module boundary, not the defining file, matches test patterns
- **Same file**: the defining file itself (excluded from usage counts)

Apply re-export detection and import-only rules per the language file.

> **False positives.** Short or generic names produce bogus matches. Apply strict qualified-path matching (discard bare occurrences) when **either** condition is true: (a) the name is ≤ 4 characters, **or** (b) the name is in this list: `new`, `parse`, `get`, `set`, `from`, `into`, `run`, `call`, `send`, `open`, `read`, `write`, `close`, `init`, `start`, `stop`, `next`, `hash`, `eq`, `cmp`, `len`, `sum`, `map`, `fmt`, `drop`, `clone`, `copy`, `Error`, `Result`, `Option`, `Config`, `Handler`, `Builder`, `Client`, `Server`, `Manager`, `Factory`, `Provider`, `Adapter`, `Wrapper`, `Reader`, `Writer`, `Stream`, `Buffer`, `Message`, `Request`, `Response`, `Context`, `Module`, `id`, `name`, `type`, `value`, `data`, `info`, `status`, `error`, `args`, `params`, `that`, `other`, `Default`, `Debug`, `Display`, `Hash`, `PartialEq`, `Eq`, `Ord`, `PartialOrd`, `Serialize`, `Deserialize`, `Any`, `Base`, `Meta`, `Entity`, `Node`, `Item`, `Entry`. Qualified paths per language file.

If qualified-path matching discards all matches for a short/generic name but the raw match count before filtering was > 0, set Preliminary to `review` for that item — the zero-count is unreliable.

## 3. Return

Output is a single contiguous response containing, in order: (1) per-item blocks, then (2) the summary block. Both sections must appear in the same response — never emit one without the other.

Do **not** emit per-item blocks for `Preliminary: keep-public` items. Omit them entirely; report only their count in the summary. Emit per-item blocks only for candidates and `review` items.

### Per-item block

```
---ITEM---
Item: <name>
File: <relative_path:line>
Visibility: <keyword, e.g. pub, pub(crate), export, public, internal>
External Usages: <count> - <comma-separated paths, or "none">
Internal Usages (same file): <count>
Internal Usages (other files, non-test): <count> - <comma-separated paths, or "none">
Internal Usages (other files, test): <count> - <comma-separated paths, or "none">
Test Usages (external): <count> - <comma-separated paths, or "none">
Used In Tests Only: yes | no
Preliminary: keep-public | candidate-high | candidate-medium | review
Restriction Hint: none | can-be-private | can-be-package-private | can-be-internal | can-be-pub-super | can-be-pub-in(<path>)
---END---
```

`Used In Tests Only`: `yes` only when there is at least one test-file reference outside the module boundary AND zero non-test references from outside the module boundary. If zero references exist from outside the module boundary (both test and non-test), this is `no`.

Preliminary classification (first match wins):

1. Language is Kotlin AND visibility is `internal` AND external usages > 0 → `review`
2. External usages > 0 in non-test code → `keep-public`
3. Used In Tests Only == yes → `candidate-medium`
4. Language is Kotlin AND visibility is `internal` AND external usages == 0 AND internal usages in other files (non-test) > 0 → `keep-public` (already at narrowest cross-file scope)
5. External usages == 0 AND internal usages in other files (non-test) == 0 → `candidate-high` (no production code depends on this item)
6. External usages == 0 AND internal usages in other files (non-test) > 0 → `candidate-medium` (can restrict scope)
7. Unclear → `review`

Restriction hint — first match wins:

1. External usages == 0 AND internal usages (other files, non-test) == 0 → `can-be-private`
2. External usages == 0 AND language is Go → `can-be-private` (unexported items visible within same package)
3. External usages == 0 AND language is Java AND current visibility is `public` or `protected` AND internal usages (other files, non-test) > 0 → `can-be-package-private`
4. External usages == 0 AND language is Kotlin AND current visibility is `public` or `protected` (not already `internal`) AND internal usages (other files, non-test) > 0 → `can-be-internal`
5. External usages == 0 AND language is Rust AND every non-test internal-usage file resides in the item's parent directory → `can-be-pub-super`
6. External usages == 0 AND language is Rust AND every non-test internal-usage file resides under a common ancestor directory strictly below the crate root → `can-be-pub-in(<ancestor_path_relative_to_crate_root>)`
7. Otherwise → `none`

### Summary block

```
---SUMMARY---
Module: <target_path>
Language: <language>
Tool Used: <cargo-public-api | grep>
Tool Available: yes (cargo-public-api succeeded) | no (cargo-public-api failed or not installed) | n/a (non-Rust language)
Total Public Items: <count>
Keep Public (not returned): <count>
Zero External Usage: <count>
Test-only External usage: <count>
Enumeration: full | truncated (<enumerated> of <total> items)
---END---
```

# Language-Specific Rules

Language files are under `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/config/agent/_audit/`. Read the matching file before enumeration:

- Rust → `lang-rust.txt`
- TypeScript/JS → `lang-typescript.txt`
- Python → `lang-python.txt`
- Go → `lang-go.txt`
- Java → `lang-java.txt`
- Kotlin → `lang-kotlin.txt`

Enumerate anything not fully private per the language file.
