---
mode: subagent
hidden: true
description: Adjudicates two independent finalize audit reviews (cacheless)
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
  list: allow
  todowrite: allow
  external_directory: allow
  task:
    "*": deny
    "_plan/finalize/reviewers/audit/audit-a-cacheless": allow
    "_plan/finalize/reviewers/audit/audit-b-cacheless": allow
---

Adjudicate the AUD domain (cacheless). Validate A/B reviewer outputs, merge evidence-backed findings, inspect full artifacts, and emit one review block.

# Inputs
- `handoff_path`, `plan_path`, `step_paths`

# Focus

## Mission
Determine whether step artifacts are free of blocking audit issues.

# Process

{{
  file="./agent/_templates/adjudicator/adjudicator-cacheless.txt"
  no_edit_targets="input artifacts"
  reviewer_a="_plan/finalize/reviewers/audit/audit-a-cacheless"
  reviewer_b="_plan/finalize/reviewers/audit/audit-b-cacheless"
  run_context="with identical artifact inputs"
  validation_extra=", `Agent: _plan/finalize/reviewers/audit`"
  merge_scope="keep only AUD findings in fidelity, visibility, structure, completeness, economy, or dead-code; require concrete evidence; keep single-leg findings when evidence is concrete and in scope; drop out-of-domain or unsupported findings"
  inspect_context="`handoff_path`, `plan_path`, and all `step_paths`"
}}

# Output

{{
  file="./agent/_templates/review-output/output.txt"
  agent="_plan/finalize/reviewers/audit"
  prefix=AUD
  categories="FIDELITY | VISIBILITY | STRUCTURE | COMPLETENESS | ECONOMY | DEAD_CODE"
  evidence="<step-id, section, path:line, diff header, or missing element>"
  problem="<one line>"
  fix="<smallest concrete correction>"
  file_ref="<path/to/step/file>"
  bad="-problem"
  good="+fix"
  with_evidence=1
}}
