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
{{ file="./agent/_implement/plan-reviewer/shared-pre.txt" }}
{{
  file="./agent/_implement/_shared/cacheless.txt"
  read_context="Read all files listed in the handoff index's File column in one batch."
  diff_target="the full handoff"
  evidence=step-id
}}
