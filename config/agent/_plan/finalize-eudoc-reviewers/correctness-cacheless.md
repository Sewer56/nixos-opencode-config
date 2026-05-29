---
mode: subagent
hidden: true
description: Reviews D# steps for coverage, specificity, and broken links (cacheless)
model: sewer-axonhub/step-3.7-flash
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

{{ file="./agent/_plan/finalize-eudoc-reviewers/_templates/correctness-header.txt" }}

# Process

{{
  file="./agent/_templates/review-process/cacheless.txt"
  read_context="Read all D# step files and relevant handoff mappings."
}}

# Output

{{
  file="./agent/_templates/review-output/output.txt"
  agent="_plan/finalize-eudoc-reviewers/correctness-cacheless"
  prefix=EDOC
  categories="COVERAGE | BROKEN_LINK"
  detail="E_CONTRADICTION | E_UNSPECIFIC | E_MISSING_DOCS | E_FROZEN_REGIONS | E_BROKEN_LINK"
  evidence="<D# step, `path:line`, or cross-step reference>"
  problem="<what is wrong>"
  fix="<smallest concrete correction>"
  file_ref="<path/to/step/file>"
  bad="-issue"
  good="+fix"
  with_detail=1
  mode=cacheless
  verified_ref="<D#>: <item description — unchanged items that remain verified>"
}}
- Target diffs to the affected D# step file.
