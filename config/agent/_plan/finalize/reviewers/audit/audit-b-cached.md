---
mode: subagent
hidden: true
description: Independent audit reviewer B (cached) for finalize adjudication
model: sewer-axonhub/deepseek-v4-pro # HIGH
temperature: 0.7  # reviewer B
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*": deny
    "*PROMPT-PLAN*.review-audit.b*.md": allow
  glob: allow
  grep: allow
  list: allow
  todowrite: allow
  external_directory: allow
---
{{ file="./agent/_plan/finalize/reviewers/audit/_templates/body.txt" mode=cached }}
