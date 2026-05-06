---
mode: subagent
hidden: true
description: Independent implementation-plan reviewer A (cached)
model: sewer-axonhub/GLM-5.1  # HIGH
temperature: 1.0  # reviewer A
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  grep: allow
  glob: allow
  bash: allow
  edit:
    "*PROMPT-PLAN*.review-implementation.md": allow
    "*PROMPT-PLAN*.review-implementation.actions.*.md": allow
    "*PROMPT-PLAN*.review-implementation.a.md": allow
    "*PROMPT-PLAN*.review-implementation.a.actions.*.md": allow
  list: allow
  todowrite: allow
  external_directory: allow
---
{{ file="./agent/_implement/plan-reviewer/_templates/body.txt" mode=cached }}
