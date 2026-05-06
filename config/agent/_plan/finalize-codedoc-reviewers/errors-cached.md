---
mode: subagent
hidden: true
description: Checks code-adjacent error documentation coverage and specificity for finalized steps (cached)
model: sewer-axonhub/MiniMax-M2.7  # LOW
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
  edit:
    "*PROMPT-PLAN*.review-codedoc-errors.md": allow
  external_directory: allow
---

{{
  file="./agent/_plan/finalize-codedoc-reviewers/_templates/errors-header.txt"
  description="Review step artifacts' code-adjacent error documentation."
  inputs="- `handoff_path` (e.g., `<artifact_base>.handoff.md`)\n- `plan_path` (e.g., `<artifact_base>.draft.md`)\n- `step_pattern` (e.g., `<artifact_base>.step.*.md`)"
  focus_file="errors-focus.plan.md"
}}

# Process

 {{
  file="./agent/_templates/review-process/cached.txt"
  has_cache_derivation=1
  delta_source=handoff_path
  cache_derivation="replace `.handoff.md` with `.review-codedoc-errors.md`"
  reads_review_ledger=1
  preserve_byte_exact=1
}}

# Output

{{
  file="./agent/_plan/finalize-codedoc-reviewers/_templates/errors-output.txt"
  mode=cached
  variant=codedoc
  agent_name="_plan/finalize-codedoc-reviewers/errors-cached"
  err_prefix=CERR
  evidence1="<section, `path:line`, or missing element>"
  file_ref="<path/to/step/file>"
  diff_target_note=" targeting the affected step file with the exact `# Errors` section to add or fix"
  verified_ref="<I#/T#>: <item description — unchanged items that remain verified>"
}}