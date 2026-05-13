---
mode: subagent
hidden: true
description: Reviews end-user documentation for wording quality — sentence flow, passive voice, filler, and wordiness (cached)
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
    "*PROMPT-DOCS-*.review-wording*.md": allow
  grep: allow
  glob: allow
  list: allow
  todowrite: allow
  external_directory: allow
---

{{ file="./agent/_docs/reviewers/_templates/wording-header.txt" }}

# Process

{{
  file="./agent/_templates/review-process/cached.txt"
  has_cache_derivation=1
  delta_source=handoff_path
  cache_derivation="replace the `.handoff.md` suffix with `.review-wording.md`"
  cache_record_type="per target file"
  reads_change_plan=1
  has_frozen_regions=1
  show_cache_update_detail=1
  pruned_unit="file entries"
}}

# Output

{{
  file="./agent/_docs/reviewers/_templates/shared-output.txt"
  mode=cached
  agent_name="_docs/reviewers/wording-cached"
  finding_prefix=WRD
  categories="SENTENCE_FLOW | PASSIVE_VOICE | FILLER | WORDINESS | TERMINOLOGY_CONSISTENCY | PARAGRAPH_LENGTH"
  evidence_ref="<section, `path:line`, or field>"
  problem_template="<what wording issue degrades readability>"
  fix_template="<concise replacement>"
  file_ref="<path/to/documentation/file>"
  bad_example="-wordy or awkward phrasing"
  good_example="+concise replacement"
  reviewer=wording
}}