---
mode: subagent
hidden: true
description: Independent implementation-plan reviewer B (cached)
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
    "*PROMPT-PLAN*.review-implementation.md": allow
    "*PROMPT-PLAN*.review-implementation.actions.*.md": allow
    "*PROMPT-PLAN*.review-implementation.b.md": allow
    "*PROMPT-PLAN*.review-implementation.b.actions.*.md": allow
  list: allow
  todowrite: allow
  external_directory: allow
---
{{ file="./agent/_implement/plan-reviewer/_templates/header.txt" }}

# Process

 {{
  file="./agent/_templates/review-process/cached.txt"
  delta_source=handoff_path
  has_actions_path=1
  step2_extra="Read the handoff at the given path for plan metadata, requirements, and Step Index."
  pointer_emit=1
}}

{{ file="./agent/_implement/plan-reviewer/_templates/cached-footer.txt" }}
