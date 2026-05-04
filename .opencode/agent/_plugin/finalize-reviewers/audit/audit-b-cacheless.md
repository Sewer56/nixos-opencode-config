---
mode: subagent
hidden: true
description: Independent plugin finalize audit reviewer B (cacheless)
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
{file:./agent/_plugin/finalize-reviewers/audit/shared-pre.txt}
{file:./agent/_plugin/finalize-reviewers/audit/shared-cacheless.txt}
