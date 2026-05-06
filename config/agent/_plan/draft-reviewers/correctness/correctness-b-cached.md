---
mode: subagent
hidden: true
description: Independent correctness reviewer B (cached) for plan draft adjudication
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
  edit:
    "*PROMPT-PLAN*.draft.review-correctness.md": allow
    "*PROMPT-PLAN*.draft.review-correctness.actions.*.md": allow
    "*PROMPT-PLAN*.draft.review-correctness.b.md": allow
    "*PROMPT-PLAN*.draft.review-correctness.b.actions.*.md": allow
  grep: allow
  glob: allow
  list: allow
  todowrite: allow
  external_directory: allow
---
{{ file="./agent/_plan/draft-reviewers/correctness/_templates/body.txt" mode=cached }}
