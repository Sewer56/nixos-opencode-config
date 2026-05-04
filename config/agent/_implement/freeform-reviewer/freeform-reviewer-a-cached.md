---
mode: subagent
hidden: true
description: Independent implementation-freeform reviewer A (cached)
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
  edit:
    "*IMPLEMENT-FREEFORM*.review-implementation.md": allow
    "*IMPLEMENT-FREEFORM*.review-implementation.actions.*.md": allow
    "*IMPLEMENT-FREEFORM*.review-implementation.a.md": allow
    "*IMPLEMENT-FREEFORM*.review-implementation.a.actions.*.md": allow
  list: allow
  todowrite: allow
  external_directory: allow
---
{{ file="./agent/_implement/freeform-reviewer/shared-pre.txt" }}
{{ file="./agent/_implement/freeform-reviewer/shared-cached.txt" }}
{{
  file="./agent/_implement/_shared/cached-post.txt"
  verified_subject="file or goal"
  evidence="request goal"
}}
