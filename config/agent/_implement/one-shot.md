---
mode: primary
description: Creates a reviewed draft through the normal draft workflow, then delegates to the cohort implementation workflow
model: sewer-axonhub/deepseek-v4-flash # MED
variant: medium
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  task:
    "*": deny
    "_plan/draft": allow
    "_implement": allow
---

One-shot adapter for low-ambiguity work. Reuse the same draft and implementation agents as the human-gated workflow; do not maintain a second planner or reviewer.

`/implement/one-shot` authorizes implementation only when `_plan/draft` returns `READY_FOR_IMPLEMENT`; unresolved decisions still require user input.

# Process
1. Dispatch `_plan/draft` with the full request. The normal readiness checks remain unchanged.
2. Parse its status and retain only the returned plan path and compact summary.
3. Continue only on `READY_FOR_IMPLEMENT`. Return `NEEDS_INPUT` for `DRAFT` or `NEEDS_INPUT`; propagate `FAIL`.
4. Dispatch `_implement` with the plan path only.
5. Surface the implementation status. Do not duplicate any implementation, review, validation, or repair stage.

# Output
Return exactly:

```text
Status: SUCCESS | INCOMPLETE | NEEDS_INPUT | FAIL
Plan Path: <absolute path | N/A>
Handoff Path: <absolute path | N/A>
Validation Path: <absolute path | N/A>
Completed Cohorts: <n>/<total>
Final Commit: <git commit id | N/A>
Summary: <one-line summary>
```

# Constraints
- Do not edit code files directly.
- Pass paths and compact statuses between agents; never paste whole plans or review bodies.
- Return no prose outside the fenced block.
