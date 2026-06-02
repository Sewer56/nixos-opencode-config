---
mode: subagent
hidden: true
description: Independent correctness reviewer B (cached) for plan draft adjudication
model: sewer-axonhub/deepseek-v4-pro # HIGH
temperature: 0.7  # reviewer B
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
    "*PROMPT-PLAN*.draft.review-correctness.b*.md": allow
  grep: allow
  glob: allow
  list: allow
  todowrite: allow
  external_directory: allow
---
{{ file="./agent/_plan/draft/reviewers/correctness/_templates/body.txt" mode=cached }}
