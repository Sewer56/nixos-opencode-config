---
mode: subagent
hidden: true
description: Reviews documentation coverage, inline comments, and readability for source files (cached)
model: sewer-axonhub/MiniMax-M2.7  # LOW
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
    "*PROMPT-DOC-COVERAGE-*.review-docs-readability.md": allow
  external_directory: allow
---

{{
  file="./agent/_shared/code-doc-reviewers/docs-readability-header.txt"
  description="Review source files for documentation coverage, specificity, fidelity, inline comments, and readability."
  inputs="- `handoff_path`"
  doc_domain=DDOC
  read_domain=DREAD
  doc_focus_file="documentation-focus.source.md"
}}

# Process

 {{
  file="./agent/_templates/review-process/cached.txt"
  has_cache_derivation=1
  delta_source=handoff_path
  cache_derivation="replace `.handoff.md` with `.review-docs-readability.md`"
  reads_review_ledger=1
  preserve_byte_exact=1
}}

# Output

{{
  file="./agent/_shared/code-doc-reviewers/docs-readability-output.txt"
  mode=cached
  variant=refactor
  agent_name="_refactor/document-reviewers/docs-and-readability-cached"
  doc_domain=DDOC
  read_domain=DREAD
  evidence1="<`path:line`, or missing element>"
  evidence2="<`path:line`, or field>"
  file_ref="<path/to/source/file>"
  read_categories="D_UNDEFINED_JARGON | D_AMBIGUOUS_LANGUAGE | D_COMPOUND_TERM_COMPRESSION | D_OPAQUE_REFERENCE | D_ACRONYM_WITHOUT_EXPANSION | D_SENTENCE_FLOW | D_PASSIVE_VOICE | D_FILLER | D_WORDINESS | D_TERMINOLOGY_CONSISTENCY"
  target_type="source file"
  leave_scope="`# Errors` completeness and implementation correctness"
  doc_constraint_extra=" and missing required inline comments in non-trivial changed bodies"
  verified_ref="<path>: <item description — unchanged items that remain verified>"
}}