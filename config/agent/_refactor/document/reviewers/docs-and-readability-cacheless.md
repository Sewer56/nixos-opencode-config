---
mode: subagent
hidden: true
description: Reviews documentation coverage, inline comments, and readability for source files (cacheless)
model: sewer-axonhub/step-3.7-flash  # LOW
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

{{
  file="./agent/_plan/finalize/codedoc-reviewers/_templates/docs-readability-header.txt"
  description="Review source files for documentation coverage, specificity, fidelity, inline comments, and readability."
  variant=refactor
  doc_domain=DDOC
  read_domain=DREAD
}}

# Process

{{
  file="./agent/_templates/review-process/cacheless.txt"
  read_context="Load `handoff_path`."
  reads_review_ledger=1
  reads_decisions=1
}}

# Output

{{
  file="./agent/_plan/finalize/codedoc-reviewers/_templates/docs-readability-output.txt"
  mode=cacheless
  variant=refactor
  agent_name="_refactor/document/reviewers/docs-and-readability-cacheless"
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
