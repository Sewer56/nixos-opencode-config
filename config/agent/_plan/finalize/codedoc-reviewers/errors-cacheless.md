---
mode: subagent
hidden: true
description: Checks code-adjacent error documentation coverage, specificity, readability, and wording for finalized steps (cacheless)
model: sewer-axonhub/step-3.7-flash  # LOW
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

{{
  file="./agent/_plan/finalize/codedoc-reviewers/_templates/errors-header.txt"
  description="Review step artifacts' code-adjacent error documentation for coverage, specificity, fidelity, readability, and wording."
  variant=codedoc
}}

# Process

{{
  file="./agent/_templates/review-process/cacheless.txt"
  read_context="Load `handoff_path` sections."
}}

# Output

{{
  file="./agent/_plan/finalize/codedoc-reviewers/_templates/errors-output.txt"
  mode=cacheless
  variant=codedoc
  agent_name="_plan/finalize/codedoc-reviewers/errors-cacheless"
  err_prefix=CERR
  evidence1="<section, `path:line`, or missing element>"
  file_ref="<path/to/step/file>"
  diff_target_note=" targeting the affected step file with the exact `# Errors` section to add or fix"
  verified_ref="<I#/T#>: <item description — unchanged items that remain verified>"
}}
