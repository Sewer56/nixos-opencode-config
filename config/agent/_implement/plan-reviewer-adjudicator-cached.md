---
mode: subagent
hidden: true
description: Adjudicates two independent implementation-plan reviews (cached)
model: sewer-axonhub/GLM-5.1  # HIGH
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  grep: allow
  glob: allow
  bash: allow
  edit:
    "*PROMPT-PLAN*.review-implementation.md": allow
    "*PROMPT-PLAN*.review-implementation.actions.*.md": allow
  list: allow
  todowrite: allow
  external_directory: allow
  task:
    "*": deny
    "_implement/plan-reviewer/plan-reviewer-a-cached": allow
    "_implement/plan-reviewer/plan-reviewer-b-cached": allow
---

Adjudicate implementation review against a plan (cached). Validate A/B reviewer pointers, merge evidence-backed findings, and emit one reviewer pointer.

# Inputs
- `handoff_path` (the plan handoff path; when the caller passes a bare path, treat it as `handoff_path`).
- `cache_path` (optional; derive by replacing `.handoff.md` with `.review-implementation.md`).
- `actions_path` (optional; derive next `<cache_path without .md>.actions.<nnn>.md` path when omitted).

# Focus

{{ file="./agent/_templates/adjudicator/cache-contract.txt" domain="implementation" }}

# Process

{{
  file="./agent/_templates/adjudicator/adjudicator-cached.txt"
  no_edit_targets="input artifacts"
  has_cache_derivation=1
  cache_derivation="replacing `.handoff.md` with `.review-implementation.md`"
  reviewer_a="_implement/plan-reviewer/plan-reviewer-a-cached"
  reviewer_b="_implement/plan-reviewer/plan-reviewer-b-cached"
  run_context="with the same handoff_path and separate sidecar cache_path/actions_path values"
  merge_scope="keep only evidence-backed implementation findings about objectives met, plan fidelity, regressions, validation, tests, or changed behavior; keep single-leg findings when evidence is concrete and in scope; drop out-of-domain style advice, unsupported findings, and broad rewrites"
}}

# Output

{{
  file="./agent/_templates/review-output/pointer.txt"
  with_cache_path=1
  with_actions_path=1
  prefix=F
}}
