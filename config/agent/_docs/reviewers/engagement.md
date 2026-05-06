---
mode: subagent
hidden: true
description: Reviews end-user documentation for reader engagement and structural quality
model: sewer-axonhub/MiniMax-M2.7  # LOW
reasoningEffort: medium
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*PROMPT-DOCS-*.review-engagement.md": allow
  grep: allow
  glob: allow
  list: allow
  todowrite: allow
  external_directory: allow
---

Review end-user documentation for reader engagement and structural quality.

# Inputs

- `handoff_path` (`<artifact_base>.handoff.md`) — contains `## Change Plan` with per-file scope levels and frozen regions.

# Focus

(Principles distilled from landing-page and copywriting research — baked in, no external reading required.)

Consider page type (landing, getting-started, guide, reference, changelog, migration guide) when applying checks.

{{ file="./rules/eudoc-review/engagement.md" }}

# Process

 {{
  file="./agent/_templates/review-process/cached.txt"
  has_cache_derivation=1
  delta_source=handoff_path
  cache_derivation="replace the `.handoff.md` suffix with `.review-engagement.md`"
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
  agent_name="_docs/reviewers/engagement"
  finding_prefix=ENG
  categories="HOOK_FIRST | SHOW_DONT_TELL | SCANNABILITY | PROGRESSIVE_COMPLEXITY | NO_FLUFF | QUICK_START"
  evidence_ref="<section, `path:line`, or structural pattern>"
  problem_template="<what engagement or structural issue degrades the reader experience>"
  fix_template="<smallest concrete correction>"
  file_ref="<path/to/documentation/file>"
  bad_example="-engagement issue"
  good_example="+corrected structure or content"
  block_rule="missing hooks on landing pages, missing concrete examples on getting-started/guide pages, fluff, and progressive-complexity violations"
  allow_rule="reference-page hook issues, scannability on non-landing pages, or minor engagement concerns"
  reviewer=engagement
}}