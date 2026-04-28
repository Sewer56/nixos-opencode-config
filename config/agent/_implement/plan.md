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
    "*PROMPT-PLAN.*.*.md": deny
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

Implements a finalized machine plan with automated review.

# Prerequisite
`PROMPT-PLAN.handoff.md` must exist (produced by `/plan/finalize`).

# Steps

## 1. Load
Read `PROMPT-PLAN.handoff.md` for metadata and Step Index. For each step file listed in the Step Index's File column, extract `Lines:` ranges and compute read parameters before issuing any read call:

1. Parse each `Lines: ~<start>-<end>` range
2. Compute: `offset = <start>`, `limit = <end> - <start> + 1`
3. Issue read with those `offset`/`limit` values only

Full-file reads on files carrying `Lines:` ranges are prohibited.

## 2. Implement
Apply steps in Step Index order. After each cohesive group of changes: format, lint, build, test. Iterate until all checks pass.

## 3. Review loop
Spawn `@_implement/plan-reviewer`, passing the handoff path. If findings (BLOCKING or ADVISORY): fix all, re-review. Repeat until `Decision: PASS` or 5 total iterations.

## 4. Report
Return exactly:
Status: SUCCESS | INCOMPLETE | FAIL
Iterations: <n>
Summary: <one-line summary>
