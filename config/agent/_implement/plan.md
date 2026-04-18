---
mode: primary
description: Implements a machine plan with an automated review loop
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*": allow
    "*PROMPT-PLAN.md": deny
    "*PROMPT-PLAN.handoff.md": deny
    "*PROMPT-PLAN.step.*.md": deny
  grep: allow
  glob: allow
  list: allow
  bash: allow
  todowrite: allow
  external_directory: allow
  task:
    "*": "deny"
    "_implement/plan-reviewer": "allow"
---

Implement a finalized machine plan with an automated review loop.

# Prerequisites
- `PROMPT-PLAN.handoff.md` must exist (from `/plan/finalize`). Plan content is in handoff; implementation/test steps are in individual files matching `PROMPT-PLAN.step.*.md`.

# Workflow

## 1. Read the machine plan
- Read `PROMPT-PLAN.handoff.md` (required) for plan metadata, requirements, and Step Index.
- It links to the source plan for additional context if needed.
- Implementation and test steps live in individual files matching `PROMPT-PLAN.step.*.md`. Read all step files in one batch.

## 2. Implement
- Follow implementation steps in order: for each I# step in the handoff's Step Index, apply the step from the corresponding file matching `step_pattern`.
- After all implementation steps, follow test steps in order: for each T# step in the Step Index, apply the step from the corresponding file matching `step_pattern`.
- Run formatter, linter, build, and tests after each cohesive group of changes.
- Iterate until all checks pass clean.

## 3. Review
- Spawn `@_implement/plan-reviewer`, passing the handoff path.
- Wait for the review packet.

## 4. Loop
- If any findings (BLOCKING or ADVISORY), fix them all and re-run the reviewer.
- Repeat until the reviewer returns `Decision: PASS` or 5 iterations are reached.

## 5. Report
- Return final status. No auto-commit.
- Never modify plan artifacts (`PROMPT-PLAN.md`, `PROMPT-PLAN.handoff.md`, files matching `PROMPT-PLAN.step.*.md`).

# Output
Return exactly:

```text
Status: SUCCESS | INCOMPLETE | FAIL
Iterations: <n>
Summary: <one-line summary>
```
