---
mode: subagent
hidden: true
description: Independent audit reviewer A (cacheless) for finalize adjudication
model: sewer-axonhub/GLM-5.1  # HIGH
temperature: 1.0  # reviewer A
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  glob: allow
  list: allow
  todowrite: allow
  external_directory: allow
---
{{ file="./agent/_plan/finalize-reviewers/audit/_templates/header.txt" }}

# Process

{{
  file="./agent/_templates/review-process/cacheless.txt"
  read_context="Read `plan_path` and all `step_paths`. Read `handoff_path` for full context."
}}

{{ file="./agent/_plan/finalize-reviewers/audit/_templates/cacheless-footer.txt" }}
