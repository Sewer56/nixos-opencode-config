---
mode: subagent
hidden: true
description: Independent correctness reviewer A (cached) for plan draft adjudication
model: sewer-axonhub/GLM-5.1  # HIGH
temperature: 1.0  # reviewer A
reasoningEffort: medium
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*PROMPT-PLAN*.draft.review-correctness.md": allow
    "*PROMPT-PLAN*.draft.review-correctness.actions.*.md": allow
    "*PROMPT-PLAN*.draft.review-correctness.a.md": allow
    "*PROMPT-PLAN*.draft.review-correctness.a.actions.*.md": allow
  grep: allow
  glob: allow
  list: allow
  todowrite: allow
  external_directory: allow
---
{{ file="./agent/_plan/draft-reviewers/correctness/_templates/header.txt" }}

# Process

 {{
  file="./agent/_templates/review-process/cached.txt"
  has_cache_derivation=1
  delta_source=draft_handoff_path
  cache_derivation="derive from artifact_base: `<artifact_base>.draft.handoff.md` → `<artifact_base>.draft.review-correctness.md`"
  cache_record_type="per `[P#]`"
  has_actions_path=1
  show_cache_format=1
  cache_format="# Review Cache: <domain>\nLatest Actions: <actions_path>\n\n## Verified Observations\n- [P#]: <grounding snapshot — one line each>\n\n## Finding Ledger\n| ID | Status | Category | Severity | Introduced In | Latest Action | Resolution |\n|---|---|---|---|---|---|---|\n| XXX-NNN | OPEN | — | — | — | — | — |"
  show_cache_update_detail=1
  pruned_unit="`[P#]` ids"
  pointer_emit=1
}}

{{ file="./agent/_plan/draft-reviewers/correctness/_templates/cached-footer.txt" }}
