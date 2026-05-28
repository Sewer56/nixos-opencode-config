---
mode: subagent
hidden: true
description: Reviews applied error docs for specificity, format, and fidelity (cached)
model: sewer-axonhub/MiniMax-M2.7  # LOW
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
  edit:
    "*": deny
    "*PROMPT-ERROR-DOCS*.md": allow
---

{{ file="./agent/_refactor/_templates/errors-reviewer-header.txt" mode=cached }}

# Process

{{
  file="./agent/_templates/review-process/cached.txt"
  delta_source=cache_path
}}

In the `# REVIEW` output, set `Agent:` to `_refactor/errors-reviewer-cached`.

# Output

{{
  file="./agent/_templates/review-output/output.txt"
  agent="_refactor/errors-reviewer-cached"
  prefix=ERR
  categories="SPECIFICITY | FORMAT | FIDELITY"
  evidence="<section, `path:line`, or missing element>"
  problem="<what is wrong>"
  fix="<smallest concrete correction>"
  file_ref="<path/to/plan/item/file>"
  bad="-+proposed error docs with vague trigger"
  good="++proposed error docs with concrete trigger"
  with_lines=1
  mode=cached
  verified_ref="<list items checked with no issues found>"
}}

- Cite source file evidence when grounding a finding.

# Constraints

- On malformed-output retry, do not re-read source files that were already analyzed.
