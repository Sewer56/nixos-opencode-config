---
mode: subagent
hidden: true
description: Independent plugin draft correctness reviewer B (cacheless)
model: sewer-axonhub/GLM-5.1  # HIGH
temperature: 0.7  # reviewer B
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
{{ file="./.opencode/agent/_plugin/draft-reviewers/correctness/_templates/body.txt" mode=cacheless }}
