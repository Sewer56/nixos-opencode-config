---
mode: subagent
hidden: true
description: Reviews D# steps for coverage, specificity, and broken links (cached)
model: sewer-axonhub/GLM-5.1
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
    "*PROMPT-PLAN*.review-eudoc-correctness.md": allow
  external_directory: allow
---

{{ file="./agent/_plan/finalize-eudoc-reviewers/_templates/correctness-header.txt" }}

## Read strategy
On first review: If Delta was passed inline, skip reading `handoff_path` — use the inline Step Index and Requirement Trace Matrix rows. Read all D# step files. For UPDATE scope: read target doc files at the line ranges the D# step specifies — do not read full target files beyond those ranges unless evidence is insufficient. For NEW: read sibling pages. Skip ARCHITECTURE.md, source code, or I#/T# step files unless a D# step explicitly references them as evidence.

On re-review: Read `## Delta` from `handoff_path` for status changes. Read ONLY D# steps marked Changed or New in Delta — skip Unchanged steps (they are in cache as Verified). Do NOT re-read the full handoff, target doc files, or sibling pages for Unchanged items.

# Process

{{
  file="./agent/_templates/review-process/cached.txt"
  has_cache_derivation=1
  delta_source=handoff_path
  cache_derivation="replace the `.handoff.md` suffix with `.review-eudoc-correctness.md`"
  cache_record_type="per item (D#)"
  has_inline_delta=1
  has_frozen_regions=1
}}

In the `# REVIEW` output, set `Agent:` to `_plan/finalize-eudoc-reviewers/correctness-cached`.

# Output

{{
  file="./agent/_templates/review-output/output.txt"
  agent="_plan/finalize-eudoc-reviewers/correctness-cached"
  prefix=EDOC
  categories="COVERAGE | BROKEN_LINK"
  detail="E_CONTRADICTION | E_UNSPECIFIC | E_MISSING_DOCS | E_FROZEN_REGIONS | E_BROKEN_LINK"
  evidence="<D# step, `path:line`, or cross-step reference>"
  problem="<what is wrong>"
  fix="<smallest concrete correction>"
  file_ref="<path/to/step/file>"
  bad="-issue"
  good="+fix"
  with_detail=1
  mode=cached
  verified_ref="<D#>: <item description — unchanged items that remain verified>"
}}

- Target diffs to the affected D# step file.
