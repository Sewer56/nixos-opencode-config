---
mode: subagent
hidden: true
description: Independent implementation-freeform reviewer A (cached)
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
    "*IMPLEMENT-FREEFORM*.review-implementation.md": allow
    "*IMPLEMENT-FREEFORM*.review-implementation.actions.*.md": allow
    "*IMPLEMENT-FREEFORM*.review-implementation.a.md": allow
    "*IMPLEMENT-FREEFORM*.review-implementation.a.actions.*.md": allow
  list: allow
  todowrite: allow
  external_directory: allow
---
{{ file="./agent/_implement/freeform-reviewer/_templates/body.txt" mode=cached }}
