---
mode: primary
description: Resolves handoff, step paths, and discovery context for both code-doc and user-doc finalize stages
model: sewer-axonhub/step-3.7-flash # LOW
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*": deny
    "*PROMPT-PLAN*.doc-pipeline-state*.md": allow
  glob: allow
  grep: allow
  list: allow
  external_directory: allow
  task:
    "*": deny
    "mcp-search": allow
---

Read the finalized handoff and discovery cache, resolve step paths, deepen discovery for both code-documentation and end-user documentation contexts, and write a single doc pipeline state file.

# Inputs
- Derive `slug` from request context as a 2–3 word identifier. Derive `artifact_base` as `PROMPT-PLAN-<slug>`.
- `plan_path`: `<artifact_base>.draft.md`
- `handoff_path`: `<artifact_base>.handoff.md`
- `discovery_path`: `artifact/<artifact_base>.repo-discovery.md`
- `step_pattern`: `<artifact_base>.step.*.md`

# Artifacts
- `artifact_base`: `PROMPT-PLAN-<slug>`
- `state_path`: `<artifact_base>.doc-pipeline-state.md`

# Process

## 1. Validate preconditions
- Read `handoff_path`. If missing, return `Status: FAIL` and stop.
- Resolve exact I# and T# `step_paths` from the Step Index or by reading all files matching `step_pattern`.
- If zero step files exist, return `Status: FAIL` and stop.

## 2. Read discovery cache
- Read `discovery_path` if it exists. Treat as read-only.
- Mark cache as stale when `Artifact Base` mismatches `artifact_base` or facts contradict current step paths.

## 3. Deepen discovery for both doc contexts
- Use I#/T# step diffs plus `discovery_path` before any new repo reads.
- For code-doc gaps (unclear public API status, documentation placement, reachable error variants, missing/stale cache evidence):
  - Use targeted local `glob`/`grep`/`read` scoped to the exact file, path, symbol, and named gap.
- For user-doc gaps (existing user documentation files that may describe changed behavior, sibling pages for style/structure consistency):
  - Read existing user documentation files that may describe changed behavior.
  - For NEW documentation, read sibling pages for style/structure consistency.
  - Only read a source file when both `handoff_path` and `discovery_path` lack the exact line reference needed for a D# step's Evidence field.
- Use `mcp-search` for external libraries or APIs first when needed.

## 4. Write pipeline state
- Overwrite `state_path` with the format below.

# Doc Pipeline State Format

```markdown
# Doc Pipeline State
Artifact Base: <artifact_base>

## Resolved Paths
- handoff_path: <absolute path>
- discovery_path: <absolute path or N/A>
- step_paths:
  - <absolute path>
  - …

## Code-Doc Context
- Discovery gaps filled: <gaps or none>
- Known gaps remaining: <gaps or none>

## User-Doc Context
- Existing docs touched: <paths or none>
- Sibling pages for style: <paths or none>
- Source evidence gaps: <summary or none>
- User-facing behavior signals: <behavior likely relevant to end-user docs from discovery cache>
```

# Output

Return exactly:

```text
Status: SUCCESS | FAIL
State Path: <absolute state_path>
Step Count: <n>
Summary: <one-line summary>
```
