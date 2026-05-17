---
mode: subagent
hidden: true
description: Reviews end-user documentation for cross-page coherence — broken links, terminology drift, and content duplication (cached)
model: sewer-axonhub/Qwen3.5-397B-A17B  # LOW
reasoningEffort: medium
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*": deny
    "*PROMPT-DOCS-*.review-consistency*.md": allow
  grep: allow
  glob: allow
  list: allow
  todowrite: allow
  external_directory: allow
---

{{ file="./agent/_docs/reviewers/_templates/consistency-header.txt" }}

# Process

{{
  file="./agent/_templates/review-process/cached.txt"
  delta_source=handoff_path
  cache_record_type="per target file pair"
  reads_change_plan=1
  single_file_pass=1
  has_frozen_regions=1
  show_cache_update_detail=1
  pruned_unit="file entries"
}}

# Output

{{
  file="./agent/_docs/reviewers/_templates/shared-output.txt"
  mode=cached
  agent_name="_docs/reviewers/consistency-cached"
  finding_prefix=CON
  categories="BROKEN_LINK | TERMINOLOGY_DRIFT | CONTENT_DUPLICATION | ORPHANED_REFERENCE"
  evidence_ref="<section, `path:line`, or cross-page reference>"
  problem_template="<what cross-page inconsistency degrades coherence>"
  fix_template="<smallest concrete correction>"
  file_ref="<path/to/documentation/file>"
  bad_example="-inconsistent or broken cross-page reference"
  good_example="+corrected reference or deduplicated content"
  reviewer=consistency
}}