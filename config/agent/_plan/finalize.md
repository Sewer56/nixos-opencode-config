---
mode: primary
description: Converts a confirmed draft plan into reviewed code and test steps
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*": deny
    "*PROMPT-PLAN*.step.*.md": allow
  todowrite: allow
  external_directory: allow
  glob: allow
  grep: allow
  task:
    "*": deny
    "_plan/finalize-review": allow
---

Orchestrate the finalize pipeline: write step files from the handoff, then dispatch review. The handoff already exists from `/plan/finalize-prep`.

# Inputs
- Derive `slug` from request context as a 2–3 word identifier. Derive `artifact_base` as `PROMPT-PLAN-<slug>`.
- Required: `<artifact_base>.pipeline-state.md` must exist from a prior `/plan/finalize-prep` run.

# Artifacts
- `artifact_base`: `PROMPT-PLAN-<slug>`
- `state_path`: `<artifact_base>.pipeline-state.md`
- `plan_path`: `<artifact_base>.draft.md`
- `handoff_path`: `<artifact_base>.handoff.md`
- `discovery_path`: `artifact/<artifact_base>.repo-discovery.md`
- `step_pattern`: `<artifact_base>.step.*.md`

# Process

## 1. Read pipeline state
- Read `state_path`.
- Fast-fail if missing or unreadable: return `Status: FAIL`.
- Derive `artifact_base`, `plan_path`, `handoff_path`, `discovery_path`, and `step_pattern` from the pipeline state.
- Extract `user_notes` from `## User Notes`.
- Read `handoff_path`. Fast-fail if missing or unreadable: return `Status: FAIL`, mention that `/plan/finalize-prep` must succeed first.
- Do not rewrite `plan_path`, `state_path`, or `handoff_path` (review phase owns Delta/Ledger updates).

## 2. Write step files
- Read `discovery_path` for file ownership, key symbols, and test file locations (treat as unavailable if missing or FAIL).
- For each step in the handoff's Step Index table, write a detailed step body file matching `step_pattern`.
- Keep the step plan concrete: an implementer must not need to invent file placement, major structure, missing test coverage, verification commands, or code shape.
- Ground each step in the current repo surface with a real file path, an anchor, repo evidence, and a short code snippet or diff.
- Stable numbering: use the I#/T# numbers from the handoff's Step Index. Do not renumber.
- Write each step to its own file as named in the Step Index.
- Collect written `step_paths` for the review stage.

## 3. Review
- Dispatch `_plan/finalize-review` with `handoff_path`, `plan_path`, collected `step_paths`, `discovery_path`, and `user_notes`.
- Propagate its `Status` and `Review Iterations`.

## 4. Return output

# Step Templates

## Implementation Step (`<artifact_base>.step.I1.md`)

```markdown
# I1: `path/to/file`

Action: UPDATE | INSERT | ADD | REMOVE
Why: <why this file changes>
Anchor: `<existing symbol or section>` | `None`
Lines: ~<start>-<end> | `None`
Insert at: before | after | replace `<anchor or region>` | `None`

Import diff:

~~~diff
<import changes or `None`>
~~~

Code Shape:

Use the target file language or `diff`. Generate code only.

Changes:
- <concrete code change>
Dependencies: None | I#
Evidence: `path/to/file:line` | `path/to/nearby/pattern:line`
```

## Test Step (`<artifact_base>.step.T1.md`)

```markdown
# T1: `path/to/test-or-module`

Action: UPDATE | INSERT | ADD | REMOVE
Purpose: <behavior to prove>
Covers: REQ-###
Anchor: `<existing symbol or section>` | `None`
Lines: ~<start>-<end> | `None`
Insert at: before | after | replace `<anchor or region>` | `None`

Import diff:

~~~diff
<import changes or `None`>
~~~

Code shape:

Use the target file language or `diff`. Generate code only.

Changes:
- <specific checks>
Parameterization: None | <cases>
Dependencies: None | I# | T#
Evidence: `path/to/file:line` | `path/to/nearby/pattern:line`
```

# Output

Return exactly:

```text
Status: SUCCESS | INCOMPLETE | FAIL
Plan Path: <absolute path to `<artifact_base>.draft.md`>
Handoff Path: <absolute path to `<artifact_base>.handoff.md`>
Step Pattern: `<artifact_base>.step.*.md`
Review Iterations: <n>
Summary: <one-line summary>
Next Command: /plan/finalize-code-docs
```

# Constraints
- Within each step file, `Lines: ~start-end` fields are approximate (±10 lines); include 2+ context lines before and after each change.
- Each diff block within a step file must carry its own `Lines: ~start-end` label (`**Lines: ~start-end**` before the diff fence). The step header `Lines: ~` lists the comma-separated union of hunk ranges. Per-hunk labels are the authoritative locators.
- Full-file `Lines:` ranges are invalid for localized changes — use only for ADD actions that add complete files.
- Nested code fences: when a fenced code block contains another fenced code block, the outer fence uses backticks (```), inner fences use tildes (~~~). Prevents premature closure of the outer block.
- Keep user-facing responses brief and factual.
