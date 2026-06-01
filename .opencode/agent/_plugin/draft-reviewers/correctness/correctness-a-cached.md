---
mode: subagent
hidden: true
description: Independent plugin draft correctness reviewer A (cached)
model: sewer-axonhub/GLM-5.1 # HIGH
temperature: 1.0  # reviewer A
variant: medium
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*": deny
    "*PROMPT-PLUGIN-PLAN*.draft.review-correctness.a*.md": allow
  grep: allow
  glob: allow
  list: allow
  todowrite: allow
  external_directory: allow
---
{{ file="./.opencode/agent/_plugin/draft-reviewers/correctness/_templates/body.txt" mode=cached }}
