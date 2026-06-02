---
mode: subagent
hidden: true
description: Cached error-doc reviewer for finalize-fast I#/T# steps
model: sewer-axonhub/deepseek-v4-flash # MED
variant: high
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
    "*": deny
    "*PROMPT-PLAN*.review-codedoc-errors*.md": allow
  external_directory: allow
---

{{
  file="./agent/_plan/finalize/codedoc-reviewers/_templates/errors-header.txt"
  description="Review step artifacts' code-adjacent error documentation for coverage, specificity, fidelity, readability, and wording."
  variant=codedoc
  mode=cached
  has_actions_path=1
}}

## Owned domain
Own error-doc findings for I#/T# step artifacts.

## Non-owned domains
Audit, tests, placement, performance, non-error code docs, and end-user docs belong to other finalize-fast reviewers. Note out-of-domain concerns briefly; do not make them blocking here.

# Process

{{
  file="./agent/_templates/review-process/cached.txt"
  delta_source=handoff_path
  has_actions_path=1
  reads_review_ledger=1
  preserve_byte_exact=1
}}

{{
  file="./agent/_templates/review-footer/cached.txt"
  agent="_plan/finalize-fast/codedoc-reviewers/errors-cached"
  domain=codedoc-errors
  ref_type=step-id
  prefix=CERR
  has_actions_path=1
  categories="COVERAGE | FORMAT | SPECIFICITY | FIDELITY | READABILITY | WORDING"
  evidence="<I#/T# step, section, path:line, or missing element>"
  problem="<one line>"
  fix="<smallest concrete correction>"
  file_ref="<path/to/step/file>"
  bad="-missing or vague error docs"
  good="+concrete # Errors docs"
  with_lines=1
  with_evidence=1
  step=""
  output_extra="- Target diffs to the affected step file with the exact `# Errors` section to add or fix.\n- Verified observations MUST include unchanged items that remain verified."
}}
