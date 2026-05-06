---
mode: subagent
hidden: true
description: Independent implementation-plan reviewer B (cacheless)
model: sewer-axonhub/GLM-5.1  # HIGH
temperature: 0.7  # reviewer B
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
  list: allow
  todowrite: allow
  external_directory: allow
---
{{ file="./agent/_implement/plan-reviewer/_templates/header.txt" }}

# Process

{{
  file="./agent/_templates/review-process/cacheless.txt"
  read_context="Read all files listed in the handoff index's File Column in one batch."
  reads_decisions=1
}}

{{ file="./agent/_implement/plan-reviewer/_templates/cacheless-footer.txt" }}
