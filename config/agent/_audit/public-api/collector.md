---
mode: subagent
hidden: true
description: Enumerates public items in one module and cross-references usage across the repo
model: sewer-axonhub/deepseek-v4-flash # MED
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

Enumerate public items in given files. Count external usages per item.

# Inputs

- `specific_paths`: list of absolute file paths (all same language)
- `language`: `rust` | `typescript` | `python` | `go` | `java` | `kotlin`
- `repo_root`: absolute path to repo root

# Workflow

## 1. Group by module

For each file in `specific_paths`, walk up to find module boundary per language file. Group files by discovered module root. Each module root becomes effective `target_path` for enumeration.

## 2. Enumerate

For each module group, enumerate public items in files of that group using grep per language file patterns. Rust: use grep (skip `cargo public-api` — partial file lists don't map to single crate).

Skip items in test files per language file's test patterns.

Record per item: name, file path, line number, visibility keyword.

## 3. Cross-reference

For each item, search entire repo for its name. Complete every item — do not skip or batch. Classify every match:

- **External**: outside module boundary, not a test file
- **Test (external)**: outside module boundary, matches test patterns
- **Internal non-test**: under module boundary, not the defining file, not a test file
- **Internal test**: under module boundary, not the defining file, matches test patterns
- **Same file**: the defining file itself (excluded from usage counts)

Apply re-export detection and import-only rules per language file.

> **False positives.** Short or generic names produce bogus matches. Apply strict qualified-path matching (discard bare occurrences) when **either** condition is true: (a) name is ≤4 characters, **or** (b) name is in this list: `new`, `parse`, `get`, `set`, `from`, `into`, `run`, `call`, `send`, `open`, `read`, `write`, `close`, `init`, `start`, `stop`, `next`, `hash`, `eq`, `cmp`, `len`, `sum`, `map`, `fmt`, `drop`, `clone`, `copy`, `Error`, `Result`, `Option`, `Config`, `Handler`, `Builder`, `Client`, `Server`, `Manager`, `Factory`, `Provider`, `Adapter`, `Wrapper`, `Reader`, `Writer`, `Stream`, `Buffer`, `Message`, `Request`, `Response`, `Context`, `Module`, `id`, `name`, `type`, `value`, `data`, `info`, `status`, `error`, `args`, `params`, `that`, `other`, `Default`, `Debug`, `Display`, `Hash`, `PartialEq`, `Eq`, `Ord`, `PartialOrd`, `Serialize`, `Deserialize`, `Any`, `Base`, `Meta`, `Entity`, `Node`, `Item`, `Entry`. Qualified paths per language file.

If qualified-path matching discards all matches for a short/generic name but raw match count before filtering was >0, set Preliminary to `review` — zero-count is unreliable.

## 4. Return

Output is single contiguous response containing, in order: (1) per-item blocks, then (2) summary block. Both sections must appear in same response — never emit one without the other.

Do **not** emit per-item blocks for `Preliminary: keep-public` items. Omit them entirely; report only their count in summary. Emit per-item blocks only for candidates and `review` items.

### Per-item block format

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

`Used In Tests Only`: `yes` only when there is at least one test-file reference outside module boundary AND zero non-test references from outside module boundary. If zero references exist from outside module boundary (both test and non-test), this is `no`.

Preliminary classification (first match wins):

1. Language is Kotlin AND visibility is `internal` AND external usages >0 → `review`
2. External usages >0 in non-test code → `keep-public`
3. Used In Tests Only == yes → `candidate-medium`
4. Language is Kotlin AND visibility is `internal` AND external usages == 0 AND internal usages in other files (non-test) >0 → `keep-public` (already at narrowest cross-file scope)
5. External usages == 0 AND internal usages in other files (non-test) == 0 → `candidate-high` (no production code depends on this item)
6. External usages == 0 AND internal usages in other files (non-test) >0 → `candidate-medium` (can restrict scope)
7. Unclear → `review`

Restriction hint — first match wins:

1. External usages == 0 AND internal usages (other files, non-test) == 0 → `can-be-private`
2. External usages == 0 AND language is Go → `can-be-private` (unexported items visible within same package)
3. External usages == 0 AND language is Java AND current visibility is `public` or `protected` AND internal usages (other files, non-test) >0 → `can-be-package-private`
4. External usages == 0 AND language is Kotlin AND current visibility is `public` or `protected` (not already `internal`) AND internal usages (other files, non-test) >0 → `can-be-internal`
5. External usages == 0 AND language is Rust AND every non-test internal-usage file resides in item's parent directory → `can-be-pub-super`
6. External usages == 0 AND language is Rust AND every non-test internal-usage file resides under a common ancestor directory strictly below crate root → `can-be-pub-in(<ancestor_path_relative_to_crate_root>)`
7. Otherwise → `none`

### Summary block format

```
---SUMMARY---
Module: <comma-separated module paths>
Language: <language>
Total Public Items: <count>
Keep Public (not returned): <count>
Zero External Usage: <count>
Test-only External usage: <count>
---END---
```

# Output

Return ONLY one fenced `text` block containing all `---ITEM---` blocks followed by exactly one `---SUMMARY---` block, using formats defined above.

# Language-Specific Rules

Lang files at `{{path:./config/agent/_audit/_templates/}}`. Read matching one before enumeration. Files:
`lang-rust.txt` `lang-typescript.txt` `lang-python.txt`
`lang-go.txt` `lang-java.txt` `lang-kotlin.txt`
Enumerate anything not fully private per language file.
