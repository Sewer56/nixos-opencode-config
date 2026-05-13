---
mode: subagent
hidden: true
description: Reviews branding for clarity — pronunciation, spelling, memorability, awkward word shape, and ambiguous meaning
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
    "*PROMPT-BRANDING*.draft.review-clarity*.md": allow
  grep: allow
  glob: allow
  list: allow
  todowrite: allow
  external_directory: allow
---

Review branding for clarity.

# Inputs

- `handoff_path` (`<artifact_base>.draft.handoff.md`) — contains `## Delta` for change tracking and `### Decisions` for cross-domain arbitration.

# Focus

## Read scope
Read `<artifact_base>.draft.md` for in-scope sections: Candidate Shortlist, Top Recommendation, Brand Positioning, Tagline and Messaging, Voice and Tone.

{{ file="./rules/groups/branding/self-clarity.md" }}

# Process

 {{
  file="./agent/_templates/review-process/cached.txt"
  has_cache_derivation=1
  delta_source=handoff_path
  cache_derivation="replace the `.handoff.md` suffix with `.review-clarity.md`"
  cache_record_type="per candidate name or brand element"
  show_cache_update_detail=1
  pruned_unit=entries
}}

# Output

{{
  file="./agent/_templates/review-output/output.txt"
  agent="_branding/reviewers/clarity"
  domains=CLR
  mode=cached
  prefix=CLR
  categories="UNPRONOUNCEABLE | AMBIGUOUS_SPELLING | POOR_MEMORABILITY | AWKWARD_WORD_SHAPE | AMBIGUOUS_MEANING_COMMON | AMBIGUOUS_MEANING_NICHE | NOT_ONE_READ_EXPLAINABLE"
  evidence="<section, `path:line`, or field>"
  problem="<what clarity issue degrades the name or brand promise>"
  fix="<concrete correction or alternative>"
  file_ref="<artifact_base>.draft.md"
  bad="-unclear name or messaging"
  good="+clearer alternative"
  with_lines=1
  verified_ref="[<ID>]: <candidate name or section — unchanged items that remain verified>"
}}

- Target diffs to `<artifact_base>.draft.md`.
