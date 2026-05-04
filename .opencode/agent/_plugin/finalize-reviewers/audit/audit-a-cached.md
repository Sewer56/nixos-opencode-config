---
mode: subagent
hidden: true
description: Independent plugin finalize audit reviewer A (cached)
model: sewer-axonhub/GLM-5.1  # HIGH
temperature: 1.0  # reviewer A
reasoningEffort: medium
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*PROMPT-PLUGIN-PLAN*.review-audit.md": allow
    "*PROMPT-PLUGIN-PLAN*.review-audit.actions.*.md": allow
    "*PROMPT-PLUGIN-PLAN*.review-audit.a.md": allow
    "*PROMPT-PLUGIN-PLAN*.review-audit.a.actions.*.md": allow
  grep: allow
  glob: allow
  list: allow
  todowrite: allow
  external_directory: allow
---
{file:./agent/_plugin/finalize-reviewers/audit/shared-pre.txt}
{file:./agent/_plugin/finalize-reviewers/audit/shared-cached.txt}
{file:./agent/_plugin/finalize-reviewers/audit/cached-post.txt}
