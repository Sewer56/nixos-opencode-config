---
mode: subagent
hidden: true
description: Independent audit reviewer A (cached) for finalize adjudication
model: sewer-axonhub/GLM-5.1  # HIGH
temperature: 1.0  # reviewer A
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*PROMPT-PLAN*.review-audit.md": allow
    "*PROMPT-PLAN*.review-audit.actions.*.md": allow
    "*PROMPT-PLAN*.review-audit.a.md": allow
    "*PROMPT-PLAN*.review-audit.a.actions.*.md": allow
  glob: allow
  list: allow
  todowrite: allow
  external_directory: allow
---
{{ file="./agent/_plan/finalize-reviewers/audit/_templates/header.txt" }}

# Process

 {{
  file="./agent/_templates/review-process/cached.txt"
  delta_source=handoff_path
  has_actions_path=1
  reads_review_ledger=1
  step2_extra="- Read `## Step Index` from `handoff_path`.\n- Use the cache's `Latest Actions` field and finding ledger for grounding."
  pointer_emit=1
}}

{{ file="./agent/_plan/finalize-reviewers/audit/_templates/cached-footer.txt" }}
