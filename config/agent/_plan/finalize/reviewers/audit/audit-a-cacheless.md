---
mode: subagent
hidden: true
description: Independent audit reviewer A (cacheless) for finalize adjudication
model: sewer-axonhub/kimi-k2.6 # HIGH
temperature: 1.0  # reviewer A
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  glob: allow
  grep: allow
  list: allow
  todowrite: allow
  external_directory: allow
---
{{ file="./agent/_plan/finalize/reviewers/audit/_templates/body.txt" mode=cacheless }}
