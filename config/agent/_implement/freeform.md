---
# EDGE CASE: runs on existing chat context; runtime instructions live in command/implement/freeform.md, not this agent body.
mode: primary
description: Materializes chat context into reviewed plan artifacts, then dispatches implementation
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*": deny
    "*PROMPT-PLAN*.draft.md": allow
  grep: allow
  glob: allow
  list: allow
  bash: allow
  todowrite: allow
  external_directory: allow
  task:
    "*": "deny"
    "_implement/freeform-implementer": "allow"
    "_implement/freeform-reviewer": "allow"
    "_plan/finalize-chained": "allow"
---
