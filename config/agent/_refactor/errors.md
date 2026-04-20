---
mode: primary
description: "Discovers, documents, and reviews error-returning functions with missing or vague # Errors documentation"
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*": allow
  write:
    "*": deny
    "*PROMPT-ERROR-DOCS.cache.md": allow
  bash: allow
  grep: allow
  glob: allow
  list: allow
  todowrite: allow
  external_directory: allow
  task:
    "*": "deny"
    "codebase-explorer": "allow"
    "_refactor/errors-collector": "allow"
    "_refactor/errors-reviewer": "allow"
---

Discover error-returning functions with missing or vague documentation. Trace error paths, draft documentation, verify coverage via collector convergence (re-spawn until stable), apply corrections, and review.

# Workflow

- `DOCUMENTATION_RULES_PATH`: `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/config/rules/documentation.md`
- `ERROR_RULES_PATH`: `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/config/rules/errors.md`
- `LANG_RULES_DIR`: `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/config/agent/_refactor`

Read `DOCUMENTATION_RULES_PATH` and `ERROR_RULES_PATH` once before starting.

## 1. Discover structure

Spawn `@codebase-explorer` to map the repository:

- Every language present
- Every module/crate boundary: Cargo.toml (Rust workspace members), package.json (Node). For Go: each directory containing `.go` files is a separate module.
- Focus on library modules and application modules. Skip test-only fixtures.

For each detected language, check if `lang-<language>-errors.txt` exists in `LANG_RULES_DIR` (defined in `# Workflow`). Only languages with a matching rules file will be processed.

## 2. Scope

Read the user message. If it contains file or directory paths, restrict collectors to those modules only. If empty or no paths found, scan the full repository.

## 3. Collect

Spawn one `@_refactor/errors-collector` per (library or application module, language) pair in a single parallel call.

Per collector, pass:

- `target_path`: absolute path to the module root
- `language`: language name as reported by `@codebase-explorer`
- `repo_root`: absolute path to the repository root
- `cache_path`: absolute path to `PROMPT-ERROR-DOCS.cache.md`

## 4. Gate

Wait for ALL collectors to return. Each collector writes its items to `cache_path` and returns `Decision: PASS`.

For each collector that reported new items (New items > 0 in its summary), re-spawn with the same `target_path`, `language`, `repo_root`, and `cache_path`. Each collector skips functions already in the cache. Wait for all re-spawned collectors to return.

- If any re-spawned collector reports new items, repeat for those collectors only.
- If all re-spawned collectors report zero new items, coverage is complete. Proceed to step 5.

**Wrong**: Waiting for one round only and proceeding regardless of whether collectors found new items.
**Correct**: Re-spawn collectors with new items, wait, repeat. Proceed only when all re-spawned collectors return zero new items.

## 5. Apply corrections

Read `PROMPT-ERROR-DOCS.cache.md`. For each item:

1. Read the source file at the path and line given.
2. If `missing`: insert the drafted error docs after existing doc sections, before the function signature.
3. If `vague`: replace the existing error docs block with the drafted content.
4. Preserve surrounding blank lines and formatting conventions.

After applying all items, run formatter, linter, build, and tests. Iterate until all checks pass clean.

## 6. Review

Spawn `@_refactor/errors-reviewer`, passing `cache_path`. Wait for the review packet.

- If findings (BLOCKING or ADVISORY): revise the applied docs in source files, update the cache, populate `## Delta` with the list of items revised in this iteration, re-run reviewer.
- Loop until no findings of any severity remain or 10 iterations.
- At cap with only ADVISORY findings: SUCCESS with risks.

## 7. Report

- Return final status. No separate apply session needed.

# Artifacts

- `cache_path`: `PROMPT-ERROR-DOCS.cache.md` (repo root)

# Output

Return exactly:

```text
Status: SUCCESS | INCOMPLETE | FAIL
Cache Path: <absolute path>
Coverage Iterations: <n>
Review Iterations: <n>
Summary: <one-line summary>
```

# Constraints

- Only write `PROMPT-ERROR-DOCS.cache.md` during this command.
- Never modify product code while collecting.
- Apply corrections only after the gate converges.
- Treat `PROMPT-ERROR-DOCS.cache.md` as read-only after the gate converges.

# Templates

## `PROMPT-ERROR-DOCS.cache.md`

````markdown
# Error Docs Cache

## Summary
- Targets: <module paths>
- Languages: <list>
- Total functions: N (M missing, K vague)
- Already specific (skipped): K
- Iteration: <n>

## Settled Facts
- [FACT-001] <fact from collector> (Source: `relative_path:line`)
- <or `None`>

## Delta
- Changed: <relative_path:line — function_name> (items revised this iteration, or `none`)

## Items

### <relative_path:line> — `function_name` (missing|vague)
````
