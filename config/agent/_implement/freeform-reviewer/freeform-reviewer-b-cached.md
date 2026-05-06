---
mode: subagent
hidden: true
description: Independent implementation-freeform reviewer B (cached)
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
  edit:
    "*IMPLEMENT-FREEFORM*.review-implementation.md": allow
    "*IMPLEMENT-FREEFORM*.review-implementation.actions.*.md": allow
    "*IMPLEMENT-FREEFORM*.review-implementation.b.md": allow
    "*IMPLEMENT-FREEFORM*.review-implementation.b.actions.*.md": allow
  list: allow
  todowrite: allow
  external_directory: allow
---
{{ file="./agent/_implement/freeform-reviewer/_templates/header.txt" }}

# Process

 {{
  file="./agent/_templates/review-process/cached.txt"
  delta_source=task_prompt
  has_actions_path=1
  step2_extra="Parse inline context from the task input. Extract `## Request`, `## Plan Summary`, `## Changes Made`, `## Notes`."
  pointer_emit=1
}}

{{ file="./agent/_implement/freeform-reviewer/_templates/cached-footer.txt" }}
