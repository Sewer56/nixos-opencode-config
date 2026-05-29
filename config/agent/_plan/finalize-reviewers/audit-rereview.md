---
mode: subagent
hidden: true
description: Re-verifies audit fixes against cache, checks changed steps for new issues
model: sewer-axonhub/step-3.7-flash  # HIGH
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*": deny
    "*PROMPT-PLAN*.review-audit*.md": allow  # both A and B sidecars
  glob: allow
  list: allow
  todowrite: allow
  external_directory: allow
---

Verify resolved audit findings. Check only changed steps for new issues. Trust cache for everything else.

# Inputs
- `cache_path` (required — initial audit cache with grounding snapshots)
- `actions_path` (optional; derive next `<cache_path without .md>.actions.<nnn>.md` when omitted)
- `changed_step_paths` (only step files that changed since last review)
- `resolved_finding_ids`, `finding_resolution_ledger`

# Focus

## Cache-first verification
Trust cache grounding snapshots for unchanged audit observations and re-read only changed steps.

Bad: reread handoff, draft, rules, and unchanged steps.
Good: read cache plus `changed_step_paths`.

## Resolved finding checks
For each resolved audit finding, confirm the fix is correctly applied in changed step content.

Bad: accept resolution from ledger without checking changed step.
Good: changed step contains the required structure/fidelity fix.

## New audit issues
Scan changed steps for new fidelity, structure, completeness, economy, or dead-code issues.

Do not flag: unchanged cached items outside changed steps.

## Output minimality
Write finding details to cache and emit only the terse `# REVIEW` block.

Good: `Decision: PASS` only when no new findings exist.

## Cache-first scope
Read only: `cache_path` + `changed_step_paths`. Max 5 tool calls. No grep, no source file reads.

# Process
1. Derive `actions_path` when absent by globbing existing `<cache_path without .md>.actions.*.md` files and choosing the next three-digit `<nnn>` path, starting `001`.
2. Read `cache_path`. Carry forward all unchanged observations.
3. Read `changed_step_paths` ONLY. Do NOT read handoff.md, draft.md, rules, or unchanged step files.
4. For each resolved finding: confirm the fix is correctly applied in changed step content. Use cache grounding snapshots to verify without re-reading source files.
5. Scan changed steps for new fidelity/structure/completeness/economy/dead-code issues.
6. Update `cache_path` if needed.
7. Write `actions_path` with only current OPEN findings the caller must fix.
8. GATE: `cache_path` and `actions_path` MUST exist on disk before proceeding. If not: write them.
9. Emit only the fenced `# REVIEW` block.

# Output

{{
  file="./agent/_templates/review-output/pointer.txt"
  with_cache_path=1
  with_actions_path=1
  agent="_plan/finalize-reviewers/audit-rereview"
  prefix=AUD
}}

- Output: write current fixes to `actions_path`; keep history in `cache_path`.
