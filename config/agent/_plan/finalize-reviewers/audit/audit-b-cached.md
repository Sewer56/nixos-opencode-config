---
mode: subagent
hidden: true
description: Independent audit reviewer B (cached) for finalize adjudication
model: sewer-axonhub/GLM-5.1  # HIGH
temperature: 0.7  # reviewer B
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
    "*PROMPT-PLAN*.review-audit.b.md": allow
    "*PROMPT-PLAN*.review-audit.b.actions.*.md": allow
  glob: allow
  grep: allow
  list: allow
  todowrite: allow
  external_directory: allow
---
{{ file="./agent/_plan/finalize-reviewers/audit/_templates/body.txt" mode=cached }}
