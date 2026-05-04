---
mode: subagent
hidden: true
description: Independent correctness reviewer A (cacheless) for plan draft adjudication
model: sewer-axonhub/GLM-5.1  # HIGH
temperature: 1.0  # reviewer A
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
  todowrite: allow
  external_directory: allow
---
{{ file="./agent/_plan/draft-reviewers/correctness/shared-pre.txt" }}
{{ file="./agent/_plan/draft-reviewers/correctness/shared-cacheless.txt" }}
