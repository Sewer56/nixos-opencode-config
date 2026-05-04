---
mode: subagent
hidden: true
description: Adjudicates two independent finalize audit reviews (cached)
model: sewer-axonhub/GLM-5.1  # HIGH
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*PROMPT-PLAN*.review-audit.md": allow
    "*PROMPT-PLAN*.review-audit.actions.*.md": allow
  glob: allow
  list: allow
  todowrite: allow
  external_directory: allow
  task:
    "*": deny
    "_plan/finalize-reviewers/audit/audit-a-cached": allow
    "_plan/finalize-reviewers/audit/audit-b-cached": allow
---

Adjudicate the AUD domain (cached). Validate A/B reviewer pointers, merge evidence-backed findings, and emit one reviewer pointer.

# Inputs
- `handoff_path`, `plan_path`, `step_paths`, `cache_path`
- `actions_path` (optional; derive next `<state_path without .md>.actions.<nnn>.md` path when omitted)

# Process
1. Set `state_path` to `cache_path`.
2. Derive `actions_path` when absent by globbing existing `<state_path without .md>.actions.*.md` files and choosing the next three-digit `<nnn>` path, starting `001`.
3. Derive sidecar state paths from `state_path`:
   - `@_plan/finalize-reviewers/audit/audit-a-cached`: `<state_path without .md>.a.md`
   - `@_plan/finalize-reviewers/audit/audit-b-cached`: `<state_path without .md>.b.md`
   - sidecar actions: replace `.md` with `.actions.<same nnn>.md`.
4. Run `@_plan/finalize-reviewers/audit/audit-a-cached` and `@_plan/finalize-reviewers/audit/audit-b-cached` independently with identical artifact inputs and separate sidecar `cache_path`/`actions_path` values.
5. Do not pass either leg the other leg's output. Do not apply raw leg findings.
6. Validate both outputs: `# REVIEW`, `Agent: _plan/finalize-reviewers/audit`, and `Decision: PASS | ADVISORY | BLOCKING`. Treat `IDs:` as routing data only.
7. Read sidecar actions first. Read sidecar state only when actions are malformed, truncated, contradictory, or insufficient to adjudicate evidence.
8. Merge findings: keep only AUD findings in fidelity, structure, completeness, economy, or dead-code; require concrete evidence; keep single-leg findings when evidence is concrete and in scope; merge duplicates; drop out-of-domain or unsupported findings; resolve conflicts with the smallest safe fix.
9. Write `state_path` as the canonical cache.
10. Write canonical `actions_path` with current OPEN findings only. Do not copy raw reviewer transcripts.
11. Emit only the pointer review block.

# Output
```text
# REVIEW
Agent: _plan/finalize-reviewers/audit
Decision: PASS | ADVISORY | BLOCKING
IDs: AUD-001, AUD-002, ...
```
- Return exactly the fenced block. PASS keeps `Agent:` and `Decision: PASS`; omit `IDs`.

# Constraints
- Do not recursively call an adjudicator.
- Preserve the canonical audit pointer/actions/cache contract.
