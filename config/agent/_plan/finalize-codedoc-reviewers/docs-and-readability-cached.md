---
mode: subagent
hidden: true
description: Reviews code-adjacent documentation in I#/T# steps for coverage, specificity, inline comments, and readability (cached)
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
    "*PROMPT-PLAN*.review-codedoc-docs-readability.md": allow
  external_directory: allow
---

{{
  file="./agent/_plan/finalize-codedoc-reviewers/_templates/docs-readability-header.txt"
  description="Review code/test steps for code-adjacent documentation coverage, specificity, fidelity, inline comments, and readability."
  variant=codedoc
  doc_domain=CDOC
  read_domain=CREAD
}}

# Process

{{
  file="./agent/_templates/review-process/cached.txt"
  has_cache_derivation=1
  delta_source=handoff_path
  cache_derivation="replace `.handoff.md` with `.review-codedoc-docs-readability.md`"
  reads_review_ledger=1
  preserve_byte_exact=1
}}

# Output

{{
  file="./agent/_plan/finalize-codedoc-reviewers/_templates/docs-readability-output.txt"
  mode=cached
  variant=codedoc
  agent_name="_plan/finalize-codedoc-reviewers/docs-and-readability-cached"
  doc_domain=CDOC
  read_domain=CREAD
  evidence1="<section, `path:line`, or missing element>"
  evidence2="<I#/T# step, section, `path:line`, or field>"
  file_ref="<path/to/step/file>"
  read_categories="C_SENTENCE_FLOW | C_PASSIVE_VOICE | C_FILLER | C_WORDINESS | C_TERMINOLOGY_CONSISTENCY | C_UNDEFINED_JARGON | C_AMBIGUOUS_LANGUAGE | C_COMPOUND_TERM_COMPRESSION | C_OPAQUE_REFERENCE | C_ACRONYM_WITHOUT_EXPANSION"
  target_type="I#/T# step file and hunk"
  doc_constraint_extra=", including missing required inline comments in non-trivial planned code diff hunks"
  verified_ref="<I#/T#>: <item description — unchanged items that remain verified>"
}}
