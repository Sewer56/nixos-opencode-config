---
mode: subagent
hidden: true
description: Independent implementation-freeform reviewer B (cacheless)
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
{{ file="./agent/_implement/freeform-reviewer/_templates/header.txt" }}

# Process

{{
  file="./agent/_templates/review-process/cacheless.txt"
  read_context="Parse inline context from the task input. Extract `## Request`, `## Plan Summary`, `## Changes Made`, `## Notes`."
}}

{{ file="./agent/_implement/freeform-reviewer/_templates/cacheless-footer.txt" }}
