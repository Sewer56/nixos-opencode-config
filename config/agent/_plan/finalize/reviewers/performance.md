---
mode: subagent
hidden: true
description: Reviews performance-sensitive finalized plan steps
model: sewer-axonhub/GLM-5.1 # HIGH
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  grep: allow
  glob: allow
  list: allow
  todowrite: allow
  external_directory: allow
---

Review only the performance-sensitive parts of step artifacts.

# Inputs

{{ file="./agent/_templates/review-inputs/plan-steps.txt" }}

# Focus

{{ file="./agent/_plan/finalize/reviewers/_templates/performance-shared-focus.txt" }}

## Read strategy
On initial review: read `handoff_path`, `plan_path`, `step_paths`, rules. Audit perf-sensitive changes. Read the `## Review Ledger` section from `handoff_path` before reviewing.

On re-review: `plan_path` is withheld. `handoff_path` is available — read only `## Delta`, `## Review Ledger`, `## Step Index`; stable sections are covered by cache. Read `changed_step_paths`. Verify resolved findings, check for new perf risks.

# Process

{{
  file="./agent/_templates/review-process/cacheless.txt"
  read_context="Read `## Delta` from `handoff_path`.\n- Read `handoff_path` for summary, requirements, Step Index, and dependency mapping.\n- Read selected exact `step_paths` in one batch."
  reads_review_ledger=1
  reads_decisions=1
}}

# Output

{{
  file="./agent/_templates/review-output/output.txt"
  agent="_plan/finalize/reviewers/performance"
  prefix=PERF
  categories="ALGORITHM | DATA | DATABASE | CONCURRENCY | VALIDATION"
  problem="<one line>"
  fix="<diff or prose>"
  file_ref="<path/to/step/file>"
  bad="-problem"
  good="+fix"
}}

- If the plan is not performance-sensitive: `Decision: PASS` with `Performance Sensitive: NO` in `## Notes`.
- If a performance finding depends on the repo surface, cite repo evidence.
- Omit the diff when the finding is a performance budget concern with no single correct implementation.
