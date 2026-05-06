---
mode: subagent
hidden: true
description: Adjudicates two independent finalize audit reviews (cached)
model: sewer-axonhub/GLM-5.1  # HIGH
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*PROMPT-PLAN*.review-audit.md": allow
    "*PROMPT-PLAN*.review-audit.actions.*.md": allow
  grep: allow
  glob: allow
  list: allow
  todowrite: allow
  external_directory: allow
  task:
    "*": deny
    "_plan/finalize-reviewers/audit/audit-a-cached": allow
    "_plan/finalize-reviewers/audit/audit-b-cached": allow
---

Adjudicate the AUD domain (cached). Validate A/B reviewer pointers, merge evidence-backed findings, and emit one reviewer pointer.

# Inputs
- `handoff_path`, `plan_path`, `step_paths`, `cache_path`
- `actions_path` (optional; derive next `<state_path without .md>.actions.<nnn>.md` path when omitted)

# Process

{{
  file="./agent/_templates/adjudicator/adjudicator-cached.txt"
  no_edit_targets="input artifacts"
  reviewer_a="_plan/finalize-reviewers/audit/audit-a-cached"
  reviewer_b="_plan/finalize-reviewers/audit/audit-b-cached"
  run_context="with identical artifact inputs and separate sidecar `cache_path`/`actions_path` values"
  validation_extra=", `Agent: _plan/finalize-reviewers/audit`"
  merge_scope="keep only AUD findings in fidelity, structure, completeness, economy, or dead-code; require concrete evidence; keep single-leg findings when evidence is concrete and in scope; drop out-of-domain or unsupported findings"
}}

# Output

{{
  file="./agent/_templates/review-output/pointer.txt"
  agent="_plan/finalize-reviewers/audit"
  prefix=AUD
}}

# Constraints
- Do not recursively call an adjudicator.
- Preserve the canonical audit pointer/actions/cache contract.
