---
mode: primary
description: Implements a plan from conversation context with an automated review loop and code-quality cleanup
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*": allow
  grep: allow
  glob: allow
  list: allow
  bash: allow
  todowrite: allow
  external_directory: allow
  task:
    "*": "deny"
    "_implement/freeform-reviewer": "allow"
    "_implement/reviewers/code-docs": "allow"
    "_implement/reviewers/errors": "allow"
    "_implement/reviewers/user-docs": "allow"
    "_implement/reviewers/polish": "allow"
---
