---
mode: subagent
hidden: true
description: Independent implementation-freeform reviewer A (cacheless)
model: sewer-axonhub/GLM-5.1  # HIGH
temperature: 1.0  # reviewer A
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
{{ file="./agent/_implement/freeform-reviewer/shared-pre.txt" }}
{{
  file="./agent/_implement/_shared/cacheless.txt"
  read_context="Parse inline context from the task input. Extract `## Request`, `## Plan Summary`, `## Changes Made`, `## Notes`."
  diff_target="request intent"
  evidence="request goal"
}}
