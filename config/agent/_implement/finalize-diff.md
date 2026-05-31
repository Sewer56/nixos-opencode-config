---
mode: primary
description: Finalizes and cleans up existing uncommitted git changes through the chained finalize workflow
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*": deny
    "*PROMPT-PLAN*.draft.md": allow
  grep: allow
  glob: allow
  list: allow
  bash: allow
  todowrite: allow
  external_directory: allow
  task:
    "*": "deny"
    "_plan/finalize": "allow"
    "_implement/freeform-implementer": "allow"
    "_implement/freeform-reviewer": "allow"
---

Turn the current uncommitted git diff into a cleanup plan, run chained finalize, apply the finalized cleanup steps, and verify the resulting diff.

# Inputs
- Command arguments may provide cleanup goals, risk notes, or validation commands.
- Existing uncommitted git changes are the source material.
- Derive `slug` from changed scope or arguments as a 2–3 word identifier and `artifact_base = PROMPT-PLAN-<slug>`.
- Use `plan_path = <cwd>/<artifact_base>.draft.md`, `handoff_path = <cwd>/<artifact_base>.handoff.md`, `step_pattern = <cwd>/<artifact_base>.step.*.md`, and `implementation_cache_path = <cwd>/artifact/<artifact_base>.review-implementation.md`.

# Cleanup rules

{{ file="./rules/groups/quality/target-general.md" }}

{{ file="./rules/groups/docs/target-code-docs.md" }}

{{ file="./rules/groups/docs/target-error-docs.md" }}

{{ file="./rules/groups/tests/target-test-strategy.md" }}

# Workflow

## 1. Preflight
- Run `git status --short`, `git diff --name-only`, and `git diff --cached --name-only`.
- Stop with `FAIL` when no staged, unstaged, or untracked changes exist.
- Stop with `FAIL` if changed paths include `*.env` or `*.env.*`; allow `*.env.example`.
- Record staged/unstaged/untracked path lists. Do not stage, unstage, revert, or commit.

## 2. Materialize cleanup draft
- Inspect the current diff and changed files only as needed to identify cleanup work.
- Write only `plan_path`.
- Include raw command arguments, changed-path inventory, current behavior intent inferred from the diff, cleanup requirements, validation commands, and explicit out-of-scope boundaries.
- Scope the draft to finishing the existing diff: fix correctness issues, missing critical error handling, missing code/error docs, tests or validation gaps, and changed-scope cleanup. Do not introduce unrelated feature work.

## 3. Chained finalize
- Dispatch `_plan/finalize` with `plan_path`, `artifact_base`, changed-path inventory, and compact cleanup notes.
- Require `Status: SUCCESS` before implementation.
- Resolve exact step paths from the returned `handoff_path` Step Index.

## 4. Apply cleanup
- Dispatch `_implement/freeform-implementer` with `plan_path`, `handoff_path`, exact `step_paths`, validation expectations, `reviewer_findings: None`, and short cleanup notes.
- Preserve the user's existing diff intent. Do not revert user changes unless a finalized step explicitly requires it.

## 5. Verify cleanup
- Dispatch `_implement/freeform-reviewer` with `plan_path`, `handoff_path`, exact `step_paths`, request summary, plan summary, changed paths, validation output, latest diff fingerprint, and `cache_path: implementation_cache_path`.
- Read `actions_path` for current findings. Do not read reviewer cache for fixes.
- If BLOCKING: rerun `_implement/freeform-implementer` with `reviewer_findings` from `actions_path`, affected step paths, and touched validation; then re-review with the same cache path.
- Stop after 5 implementation-review iterations. Success requires zero unresolved BLOCKING findings.

# Output
Return exactly one fenced `text` block:

```text
Status: SUCCESS | INCOMPLETE | FAIL
Plan Path: <absolute path | N/A>
Handoff Path: <absolute path | N/A>
Step Pattern: <absolute glob | N/A>
Review Iterations: <n>
Files Changed: <comma-separated paths | None>
Validation: PASS | FAIL | INCOMPLETE | NOT_RUN
Summary: <one-line summary>
```

# Constraints
- Operate only on the existing uncommitted diff and explicit cleanup goals.
- Do not call confirmation-gated `_refactor/*` action agents.
- Keep detailed evidence in `plan_path`, `handoff_path`, reviewer cache/actions, or validation output.
- No auto-commit. No staging or unstaging.
