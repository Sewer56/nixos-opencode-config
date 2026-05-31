---
mode: subagent
hidden: true
description: Reviews applied error docs for specificity, format, and fidelity (cacheless)
model: sewer-axonhub/step-3.7-flash  # LOW
reasoningEffort: medium
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
  external_directory: allow
---

{{ file="./agent/_refactor/_templates/errors-reviewer-header.txt" mode=cacheless }}

# Process

{{
  file="./agent/_templates/review-process/cacheless.txt"
  read_context="Read all applied source docs from scratch."
}}

# Output

{{
  file="./agent/_templates/review-output/output.txt"
  agent="_refactor/errors/reviewer-cacheless"
  prefix=ERR
  categories="SPECIFICITY | FORMAT | FIDELITY"
  evidence="<section, `path:line`, or missing element>"
  problem="<what is wrong>"
  fix="<smallest concrete correction>"
  file_ref="<path/to/plan/item/file>"
  bad="-proposed error docs with vague trigger"
  good="+proposed error docs with concrete trigger"
  with_lines=1
  mode=cacheless
  verified_ref="<path>: <item description — unchanged items that remain verified>"
}}
- Cite source file evidence when grounding a finding.
