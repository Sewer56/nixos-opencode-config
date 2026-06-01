---
mode: subagent
hidden: true
description: Reviews end-user documentation for cross-page coherence — broken links, terminology drift, and content duplication (cacheless)
model: sewer-axonhub/MiniMax-M3 # MED
variant: high
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  grep: allow
  glob: allow
  list: allow
  todowrite: allow
  external_directory: allow
---

{{ file="./agent/_docs/reviewers/_templates/consistency-header.txt" }}

# Process

{{
  file="./agent/_templates/review-process/cacheless.txt"
  read_context="Read `## Change Plan` from `handoff_path` for per-file scope levels and frozen regions."
  single_file_pass=1
}}

# Output

{{
  file="./agent/_docs/reviewers/_templates/shared-output.txt"
  mode=cacheless
  agent_name="_docs/reviewers/consistency-cacheless"
  finding_prefix=CON
  categories="BROKEN_LINK | TERMINOLOGY_DRIFT | CONTENT_DUPLICATION | ORPHANED_REFERENCE"
  evidence_ref="<section, `path:line`, or cross-page reference>"
  problem_template="<what cross-page inconsistency degrades coherence>"
  fix_template="<smallest concrete correction>"
  file_ref="<path/to/documentation/file>"
  bad_example="-inconsistent or broken cross-page reference"
  good_example="+corrected reference or deduplicated content"
}}
