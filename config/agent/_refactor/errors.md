---
mode: primary
description: "Produces a reviewed error docs plan for missing or vague # Errors documentation"
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*": deny
    "*PROMPT-ERROR-DOCS-PLAN.md": allow
  write:
    "*": deny
    "*PROMPT-ERROR-DOCS-PLAN.md": allow
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

Produce a reviewed error docs plan. Discover error-returning functions with missing or vague documentation. Draft specific error docs by tracing actual error paths. Write and review plan file until it passes. Return plan path for a separate apply session.

# ORCHESTRATION

## 1. Discover structure

Spawn `@codebase-explorer` to map the repository:

- Every language present
- Every module/crate boundary: Cargo.toml (Rust workspace members), package.json (Node). For Go: each directory containing `.go` files is a separate module.
- Focus on library modules and application modules. Skip test-only fixtures.

For each detected language, check if `lang-<language>-errors.txt` exists in `LANG_RULES_DIR` (defined in `# SHARED RULES`). Only languages with a matching rules file will be processed.

## 2. Scope

Read the user message. If it contains file or directory paths, restrict collectors to those modules only. If empty or no paths found, scan the full repository.

## 3. Collect

Spawn one `@_refactor/errors-collector` per (library or application module, language) pair in a single parallel call.

Per collector, pass:

- `target_path`: absolute path to the module root
- `language`: language name as reported by `@codebase-explorer`
- `repo_root`: absolute path to the repository root

## 4. Gate

Wait for ALL collectors to return before proceeding. Do not begin any analysis until every collector has reported.

Collector output is final — per-item blocks for missing/vague functions, then summary. Do not re-query or resume.

Verify each collector output begins with `---COLLECTOR-START---` and ends with `---COLLECTOR-END---`. If markers are missing, note it but proceed — do not re-spawn the collector.

# PLAN

## 5. Write plan file

Write `plan_path` using the template in `# Templates` below.

For every `missing` or `vague` item from the collectors, draft the proposed error documentation using the `Traced Error Paths` from the collector output.

For each item, read the matching `lang-<language>-errors.txt` from `LANG_RULES_DIR` (defined in `# SHARED RULES`). Apply:

- **Doc Format** section → write the proposed docs in the language's format
- **Zero-Path Fallback** section → apply when the collector traced zero error paths

One bullet per traced error path. Preserve the exact variant/class name from the trace. Write the trigger in plain language that a reader can predict from inputs/state alone.

Populate `## Settled Facts` from collector summaries. Populate each `## Items` subsection from collector per-item blocks.

## 6. Review loop

After writing or revising `plan_path`, spawn `@_refactor/errors-reviewer`, passing `plan_path`.

Synthesize all findings into a checklist (BLOCKING first).

If findings exist:
- Revise `plan_path` only where needed. Append one line to `## Revision History`.
- Re-run reviewer after every material revision. A material revision is any change to a **Proposed:** section or addition/removal of an item.

Loop until no findings of any severity remain or 10 iterations. If 3 consecutive iterations produce only ADVISORY findings with no BLOCKING, accept and exit early.

- No findings: SUCCESS.
- At cap with any BLOCKING finding: FAIL.
- At cap with only ADVISORY findings: SUCCESS with risks.

# SHARED RULES

- `DOCUMENTATION_RULES_PATH`: `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/config/rules/documentation.md`
- `ERROR_RULES_PATH`: `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/config/rules/errors.md`
- `LANG_RULES_DIR`: `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/config/agent/_refactor`

Read `DOCUMENTATION_RULES_PATH` and `ERROR_RULES_PATH` once before starting. `DOCUMENTATION_RULES_PATH` is source of truth for doc format and style. `ERROR_RULES_PATH` is source of truth for `# Errors` section requirements and blocking criteria.

# Artifacts

- `plan_path`: `PROMPT-ERROR-DOCS-PLAN.md` (repo root)

# Output

Return exactly:

```text
Status: SUCCESS | INCOMPLETE | FAIL
Plan Path: <absolute path>
Review Iterations: <n>
Summary: <one-line summary>
```

# Constraints

- Only write `PROMPT-ERROR-DOCS-PLAN.md` during this command.
- Never modify product code while planning.

# Templates

## `PROMPT-ERROR-DOCS-PLAN.md`

````markdown
# Error Docs Plan

## Summary
- Targets: <module paths>
- Languages: <list>
- Total functions: N (M missing, K vague)
- Already specific (skipped): K

## Settled Facts
- [FACT-001] <fact from collector> (Source: `relative_path:line`)
- <or `None`>

## Revision History
- Iteration 1: Initial draft.

## Items

### <relative_path:line> — `function_name` (missing|vague)

**Language:** <language>
**Returns:** <full return type signature>

**Current:**

<verbatim current error docs section, or "NONE">

**Traced Error Paths:**
- Variant: <Error::Foo>, Trigger: <specific condition from body>
- Variant: <Error::Bar>, Trigger: <specific condition from body>
(When zero paths traced: `Traced Error Paths: (none)`)

**Proposed:**

<drafted error docs section>

---
````
