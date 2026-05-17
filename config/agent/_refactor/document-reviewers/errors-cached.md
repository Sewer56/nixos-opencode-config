---
mode: subagent
hidden: true
description: Reviews error documentation coverage and specificity for source files (cached)
model: sewer-axonhub/Qwen3.5-397B-A17B  # LOW
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
  edit:
    "*": deny
    "*PROMPT-DOC-COVERAGE-*.review-errors*.md": allow
  external_directory: allow
---

{{
  file="./agent/_plan/finalize-codedoc-reviewers/_templates/errors-header.txt"
  description="Review source files' error documentation."
  variant=refactor
}}

# Process

{{
  file="./agent/_templates/review-process/cached.txt"
  has_cache_derivation=1
  delta_source=handoff_path
  cache_derivation="replace `.handoff.md` with `.review-errors.md`"
  reads_review_ledger=1
  preserve_byte_exact=1
}}

# Output

{{
  file="./agent/_plan/finalize-codedoc-reviewers/_templates/errors-output.txt"
  mode=cached
  variant=refactor
  agent_name="_refactor/document-reviewers/errors-cached"
  err_prefix=DERR
  evidence1="<`path:line`, or missing element>"
  file_ref="<path/to/source/file>"
  verified_ref="<path>: <item description — unchanged items that remain verified>"
}}
