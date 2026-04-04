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
    "*PROMPT-PLAN.machine.md": deny
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
- `PROMPT-PLAN.machine.md` must exist (from `/plan/finalize`).

# Workflow

## 1. Read the machine plan
- Read `PROMPT-PLAN.machine.md` (required).
- It links to the source plan and handoff for additional context if needed.

## 2. Implement
- Follow the machine plan's `## Implementation Steps` and `## Test Steps` in order.
- Run formatter, linter, build, and tests after each cohesive group of changes.
- Iterate until all checks pass clean.

## 3. Review
- Spawn `@_implement/plan-reviewer`, passing the machine plan path.
- Wait for the review packet.

## 4. Loop
- If any findings (BLOCKING or ADVISORY), fix them all and re-run the reviewer.
- Repeat until the reviewer returns `Decision: PASS` or 5 iterations are reached.

## 5. Report
- Return final status. No auto-commit.
- Never modify plan artifacts (`PROMPT-PLAN.md`, `PROMPT-PLAN.handoff.md`, `PROMPT-PLAN.machine.md`).

# Output
Return exactly:

```text
Status: SUCCESS | INCOMPLETE | FAIL
Iterations: <n>
Summary: <one-line summary>
```
