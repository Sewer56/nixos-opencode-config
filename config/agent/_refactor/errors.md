---
mode: primary
description: "Rewrites vague or missing # Errors docs with specific variant-level bullets"
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit: allow
  write: allow
  grep: allow
  glob: allow
  list: allow
  bash: allow
  todowrite: allow
  external_directory: allow
  task:
    "*": "deny"
    "codebase-explorer": "allow"
    "_refactor/errors-collector": "allow"
---

Find every public error-returning function in the repository with missing or vague error documentation. Draft specific error docs by tracing actual error paths in each function body. Present a confirmation plan before editing.

# SHARED RULES

- `DOCUMENTATION_RULES_PATH`: `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/config/rules/documentation.md`
- `LANG_RULES_DIR`: `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/config/agent/_refactor`

Read `DOCUMENTATION_RULES_PATH` once before starting. Use it as the source of truth for doc format and style.

# ORCHESTRATION

## 1. Discover structure

Spawn `@codebase-explorer` to map the repository:

- Every language present
- Every module/crate boundary: Cargo.toml (Rust workspace members), package.json (Node). For Go: each directory containing `.go` files is a separate module.
- Focus on library modules (those depended on by other modules) and application modules. Skip test-only fixtures.

For each detected language, check if `lang-<language>-errors.txt` exists in `LANG_RULES_DIR`. Only languages with a matching rules file will be processed.

## 2. Collect

Spawn one `@_refactor/errors-collector` per (library or application module, language) pair in a single parallel call.

Per collector, pass:

- `target_path`: absolute path to the module root
- `language`: language name as reported by `@codebase-explorer`
- `repo_root`: absolute path to the repository root

## 3. Gate

Wait for ALL collectors to return before proceeding. Do not begin any analysis until every collector has reported.

Collector output is final — per-item blocks for missing/vague functions, then summary. Do not re-query or resume.

# ANALYSIS

## 4. Draft

For every `missing` or `vague` item from the collectors, draft the proposed error documentation using the `Traced Error Paths` from the collector output.

For each item, read the matching `lang-<language>-errors.txt` from `LANG_RULES_DIR`. Apply:

- **Doc Format** section → write the proposed docs in the language's format
- **Zero-Path Fallback** section → apply when the collector traced zero error paths

One bullet per traced error path. Preserve the exact variant/class name from the trace. Write the trigger in plain language that a reader can predict from inputs/state alone.

## 5. Confirmation gate (REQUIRED — DO NOT SKIP)

Present the plan and STOP. Do not proceed to step 6. Use this format:

```markdown
# Proposed Error Docs Plan

Targets: <paths>

## <relative_path:line> — `function_name` (missing|vague)

**Current:**

<verbatim current error docs section, or "NONE">

**Proposed:**

<drafted error docs section>

## <relative_path:line> — `function_name` (missing|vague)

...

---

Functions already specific: N (skipped)

Say "go" to apply this plan, or suggest changes.
```

Continue ONLY when user says exactly "go".
If user suggests changes, revise the plan and re-run this gate.
DO NOT use edit or write tools before receiving "go".

# APPLY

## 6. Edit files (after `go`)

For each item in the confirmed plan:

1. Read the file.
2. Locate the function's doc comments (the line immediately before `pub fn`, `async fn`, `function`, `export function`, etc.).
3. If `missing`: insert the proposed error docs block after the existing doc sections, before the function signature.
4. If `vague`: replace the existing error docs block with the proposed block.
5. Do not alter function signatures, bodies, imports, or any non-doc content.
6. Preserve surrounding blank lines and formatting conventions in the file.

## 7. Verify

Run formatter, lint, build/type checks, and tests according to repository conventions.
Iterate until checks pass, or report exact blockers with file/function details.

## 8. Report

```
Error Docs Updated
Files: N
Functions documented: N (M missing, K vague)
Specificity: all bullets name concrete variants with trigger conditions

Top changes:
  <path:line> <function_name> — added N bullets
  ...

Verification: <pass/fail + details>
```
