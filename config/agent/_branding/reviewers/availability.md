---
mode: subagent
hidden: true
description: Reviews branding for availability — domain, package, handle, trademark, and ecosystem caveats
model: sewer-axonhub/GLM-5.1  # HIGH
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
    "*PROMPT-BRANDING*.draft.review-availability*.md": allow
  grep: allow
  glob: allow
  list: allow
  todowrite: allow
  external_directory: allow
---

Review branding for availability.

# Inputs

- `branding_path`: `<artifact_base>.draft.md`
- `handoff_path`: `artifact/<artifact_base>.draft.handoff.md`
- `cache_path` (required): `artifact/<artifact_base>.draft.review-availability.md`

# Focus

## Read scope
Read `branding_path` for in-scope sections: Candidate Shortlist, Top Recommendation, Risk and Availability Notes, Next Checks.
Cross-reference search findings from the handoff for external availability data.

{{ file="./rules/groups/branding/self-availability.md" }}

# Process

 {{
  file="./agent/_templates/review-process/cached.txt"
  delta_source=handoff_path
  cache_record_type="per candidate name or brand element"
  step2_extra="- When the reviewer's Focus includes search-findings references: also read the search findings section for external data."
  show_cache_update_detail=1
  pruned_unit=entries
}}

# Output

{{
  file="./agent/_templates/review-output/output.txt"
  agent="_branding/reviewers/availability"
  domains=AVL
  with_domains=1
  mode=cached
  prefix=AVL
  categories="DOMAIN_CAVEAT | PACKAGE_CRATE_CAVEAT | SOCIAL_HANDLE_CAVEAT | MISSING_TRADEMARK_DISCLAIMER | RISKY_AVAILABILITY_CLAIM | MISSING_ECOSYSTEM_CHECK | MISSING_NEXT_CHECKS"
  evidence="<section, `path:line`, or field>"
  problem="<what availability issue creates risk or misleading claims>"
  fix="<concrete correction or addition>"
  file_ref="<artifact_base>.draft.md"
  bad="-missing or risky availability claim"
  good="+qualified claim or disclaimer"
  with_lines=1
  verified_ref="[<ID>]: <candidate name or section — unchanged items that remain verified>"
}}

- Target diffs to `<artifact_base>.draft.md`.
