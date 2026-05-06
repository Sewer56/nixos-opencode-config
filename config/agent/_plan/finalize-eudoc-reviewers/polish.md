---
mode: subagent
hidden: true
description: Reviews D# steps for clarity, wording, engagement, and cross-page polish
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
    "*PROMPT-PLAN*.review-eudoc-polish.md": allow
  external_directory: allow
  task: deny
---

Review D# steps for clarity, wording quality, reader engagement, and cross-page polish. Domain owner for ECLR, EWRD, EENG, and ECNS findings. If only one D# step is in scope, skip cross-page checks.

# Inputs
- `handoff_path` (e.g., `<artifact_base>.handoff.md`)
- `plan_path` (e.g., `<artifact_base>.draft.md`)
- `step_pattern` (e.g., `<artifact_base>.step.*.md`)

# Focus

{{ file="./rules/eudoc-review/clarity.md" }}
{{ file="./rules/eudoc-review/wording.md" }}
{{ file="./rules/eudoc-review/engagement.md" }}
{{ file="./rules/eudoc-review/consistency.md" }}

## Plan-step context
- Do not flag frozen regions.

# Process

 {{
  file="./agent/_templates/review-process/cached.txt"
  has_cache_derivation=1
  delta_source=handoff_path
  cache_derivation="replace the `.handoff.md` suffix with `.review-eudoc-polish.md`"
  cache_record_type="per item (D#)"
  has_inline_delta=1
  has_frozen_regions=1
}}

# Output

{{
  file="./agent/_templates/review-output/output.txt"
  agent="_plan/finalize-eudoc-reviewers/polish"
  prefix=EPOL
  categories="CLARITY | WORDING | ENGAGEMENT | POLISH"
  detail="E_JARGON | E_AMBIGUOUS | E_COMPOUND | E_OPAQUE_REF | E_ACRONYM | E_PASSIVE | E_FILLER | E_WORDY | E_TERM_INCONSIST | E_PARA_LEN | E_HOOK | E_SHOW | E_SCAN | E_PROG_COMPLEX | E_FLUFF | E_QUICK_START | E_PEER_BULLET | E_BULLET_SPACE | E_TERM_DRIFT | E_DUPLICATION | E_ORPHANED"
  evidence="<D# step, `path:line`, or pattern>"
  problem="<what is wrong>"
  fix="<smallest concrete correction>"
  file_ref="<path/to/step/file>"
  bad="-issue"
  good="+fix"
  with_lines=0
  with_detail=1
  mode=cached
  verified_ref="<D#>: <item description — unchanged items that remain verified>"
  return_rule="Return ONLY the block above — no introduction, no summary, no conversational wrapper, no text before `# REVIEW` or after the final `## Notes` line."
}}

# Constraints
- **First review** (cache empty or no prior findings): If Delta was passed inline, skip reading `handoff_path` — use the inline Step Index and Requirement Trace Matrix rows. Read all D# step files in one batch. For UPDATE scope: read target doc files at the line ranges D# steps specify. Read all D# step files for cross-page polish checks (only if multiple exist). Skip ARCHITECTURE.md, source code, draft.md, or I#/T# step files unless a D# step explicitly references them.
- **Re-review** (cache has prior findings): Read `## Delta` from `handoff_path` for status changes. Read ONLY D# steps marked Changed or New in Delta — skip Unchanged steps (they are in cache as Verified). Do NOT re-read the full handoff, target doc files, or all D# steps for Unchanged items. For cross-page checks on re-review, only examine Changed D# steps against each other — skip cross-checks involving only Unchanged steps.
- Do NOT read the correctness reviewer cache (`<artifact_base>.review-eudoc-correctness.md`). Polish owns wording/clarity/engagement/consistency; correctness owns EDOC findings. If a cross-domain concern arises, note it as a short pointer in `## Notes`.
- Block for: undefined project jargon, ambiguous phrasing, compound-term compression, opaque references, project-specific acronyms, filler, passive voice in instructions, ambiguous terminology within a D# step, missing hooks on landing pages, missing examples on getting-started/guide pages, progressive-complexity violations.
- Do not block for: standard domain terms, descriptive passive voice, stylistic variation, minor wordiness, reference-page hook issues, scannability on non-landing pages, fluff, terminology drift, content duplication, orphaned references — ADVISORY only.
- Keep findings short and specific.
- Include a unified diff after every finding's `Fix:` targeting the affected D# step file.
- Follow the `# Process` section for cache, Delta, and skip handling.
- Only generate findings on in-scope D# steps. Findings on frozen regions are invalid.
