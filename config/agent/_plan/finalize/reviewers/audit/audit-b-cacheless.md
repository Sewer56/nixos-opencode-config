---
mode: subagent
hidden: true
description: Independent audit reviewer B (cacheless) for finalize adjudication
model: sewer-axonhub/glm-5.1 # HIGH
temperature: 0.7  # reviewer B
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
