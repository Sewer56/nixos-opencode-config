---
mode: subagent
hidden: true
description: Independent plugin finalize audit reviewer B (cached)
model: sewer-axonhub/glm-5.1 # HIGH
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
    "*PROMPT-PLUGIN-PLAN*.review-audit.b*.md": allow
  grep: allow
  glob: allow
  list: allow
  todowrite: allow
  external_directory: allow
---
{{ file="./.opencode/agent/_plugin/finalize-reviewers/audit/_templates/body.txt" mode=cached }}
