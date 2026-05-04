---
mode: subagent
hidden: true
description: Independent audit reviewer A (cached) for finalize adjudication
model: sewer-axonhub/GLM-5.1  # HIGH
temperature: 1.0  # reviewer A
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*PROMPT-PLAN*.review-audit.md": allow
    "*PROMPT-PLAN*.review-audit.actions.*.md": allow
    "*PROMPT-PLAN*.review-audit.a.md": allow
    "*PROMPT-PLAN*.review-audit.a.actions.*.md": allow
  glob: allow
  list: allow
  todowrite: allow
  external_directory: allow
---
{{ file="./agent/_plan/finalize-reviewers/audit/shared-pre.txt" }}
{{ file="./agent/_plan/finalize-reviewers/audit/shared-cached.txt" }}
{{ file="./agent/_plan/finalize-reviewers/audit/cached-post.txt" }}
