---
mode: subagent
hidden: true
description: Reviews documentation coverage, inline comments, and readability for source files (cached)
model: sewer-axonhub/step-3.7-flash  # LOW
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
  edit:
    "*": deny
    "*PROMPT-DOC-COVERAGE-*.review-docs-readability*.md": allow
  external_directory: allow
---

{{
  file="./agent/_plan/finalize/codedoc-reviewers/_templates/docs-readability-header.txt"
  description="Review source files for documentation coverage, specificity, fidelity, inline comments, and readability."
  variant=refactor
  mode=cached
  doc_domain=DDOC
  read_domain=DREAD
}}

# Process

{{
  file="./agent/_templates/review-process/cached.txt"
  delta_source=handoff_path
  reads_review_ledger=1
  preserve_byte_exact=1
}}

# Output

{{
  file="./agent/_plan/finalize/codedoc-reviewers/_templates/docs-readability-output.txt"
  mode=cached
  variant=refactor
  agent_name="_refactor/document/reviewers/docs-and-readability-cached"
  doc_domain=DDOC
  read_domain=DREAD
  evidence1="<`path:line`, or missing element>"
  evidence2="<`path:line`, or field>"
  file_ref="<path/to/source/file>"
  read_categories="D_UNDEFINED_JARGON | D_AMBIGUOUS_LANGUAGE | D_COMPOUND_TERM_COMPRESSION | D_OPAQUE_REFERENCE | D_ACRONYM_WITHOUT_EXPANSION | D_SENTENCE_FLOW | D_PASSIVE_VOICE | D_FILLER | D_WORDINESS | D_TERMINOLOGY_CONSISTENCY"
  target_type="source file"
  doc_constraint_extra=" and missing required inline comments in non-trivial changed bodies"
  verified_ref="<path>: <item description — unchanged items that remain verified>"
}}
