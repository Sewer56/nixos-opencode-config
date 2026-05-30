---
mode: subagent
hidden: true
description: Independent audit reviewer A (cached) for finalize adjudication
model: sewer-axonhub/GLM-5.1  # HIGH
temperature: 1.0  # reviewer A
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*": deny
    "*PROMPT-PLAN*.review-audit.a*.md": allow
  glob: allow
  grep: allow
  list: allow
  todowrite: allow
  external_directory: allow
---
{{ file="./agent/_plan/finalize-reviewers/audit/_templates/body.txt" mode=cached }}
