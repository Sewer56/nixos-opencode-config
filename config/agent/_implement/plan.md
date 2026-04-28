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
`HANDOFF_DOCUMENT` must exist (produced by `/plan/finalize`).

# Steps

## 1. Load
Read `HANDOFF_DOCUMENT` for information. For each step/rev file listed in the Step Index's File column, extract `Lines:` ranges and compute read parameters:

1. Parse each `Lines: ~<start>-<end>` range (comma-separated = multiple ranges)
2. Compute: `offset = <start>`, `limit = <end> - <start> + 1`
3. Issue all reads for the same file in parallel using those `offset`/`limit` values
4. If hunk context is insufficient, widen by ±10 lines
5. Read the full file only if widening returns no results

## 2. Implement
Apply steps in Step Index order. When a step has `Lines:` ranges, read only those ranges — never read the full file while `Lines:` ranges remain unread. After each cohesive group of changes: format, lint, build, test. Iterate until all checks pass.

## 3. Review loop
Spawn `@_implement/plan-reviewer`, passing the path to `HANDOFF_DOCUMENT`. If findings (BLOCKING or ADVISORY): fix all, re-review. Repeat until `Decision: PASS` or 5 total iterations.

## 4. Report
Return exactly:
Status: SUCCESS | INCOMPLETE | FAIL
Iterations: <n>
Summary: <one-line summary>
