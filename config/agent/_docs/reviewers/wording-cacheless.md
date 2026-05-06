---
mode: subagent
hidden: true
description: Reviews end-user documentation for wording quality — sentence flow, passive voice, filler, and wordiness (cacheless)
model: sewer-axonhub/MiniMax-M2.7  # LOW
reasoningEffort: medium
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

{{ file="./agent/_docs/reviewers/_templates/wording-header.txt" }}

# Process

{{
  file="./agent/_templates/review-process/cacheless.txt"
  read_context="Read `## Change Plan` from `handoff_path` for per-file scope levels and frozen regions."
}}

# Output

{{
  file="./agent/_docs/reviewers/_templates/shared-output.txt"
  mode=cacheless
  agent_name="_docs/reviewers/wording-cacheless"
  finding_prefix=WRD
  categories="SENTENCE_FLOW | PASSIVE_VOICE | FILLER | WORDINESS | TERMINOLOGY_CONSISTENCY | PARAGRAPH_LENGTH"
  evidence_ref="<section, `path:line`, or field>"
  problem_template="<what wording issue degrades readability>"
  fix_template="<concise replacement>"
  file_ref="<path/to/documentation/file>"
  bad_example="-wordy or awkward phrasing"
  good_example="+concise replacement"
  block_rule="filler, passive voice in instructional steps, and genuinely ambiguous terminology inconsistencies within a single page"
  allow_rule="stylistic terminology variation, descriptive passive voice, or minor wordiness"
  reviewer=wording
}}