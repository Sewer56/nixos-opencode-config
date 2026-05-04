---
mode: subagent
hidden: true
description: Adjudicates two independent plugin draft correctness reviews (cached)
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
    "*PROMPT-PLUGIN-PLAN*.draft.review-correctness.md": allow
    "*PROMPT-PLUGIN-PLAN*.draft.review-correctness.actions.*.md": allow
  grep: allow
  glob: allow
  list: allow
  todowrite: allow
  external_directory: allow
  task:
    "*": deny
    "_plugin/draft-reviewers/correctness/correctness-a-cached": allow
    "_plugin/draft-reviewers/correctness/correctness-b-cached": allow
---

Adjudicate the plugin COR domain (cached). Validate A/B reviewer pointers, merge evidence-backed findings, and emit one reviewer pointer.

# Inputs
- `context_path`, `draft_handoff_path`
- `cache_path` (optional; normal review state)
- `actions_path` (optional)

# Process
1. Derive canonical `cache_path` when absent by replacing `.draft.handoff.md` with `.draft.review-correctness.md`.
2. Set `state_path` to `cache_path`.
3. Derive `actions_path` when absent by globbing existing `<state_path without .md>.actions.*.md` files and choosing the next three-digit `<nnn>` path, starting `001`.
4. Derive sidecar state paths from `state_path`:
   - `@_plugin/draft-reviewers/correctness/correctness-a-cached`: `<state_path without .md>.a.md`
   - `@_plugin/draft-reviewers/correctness/correctness-b-cached`: `<state_path without .md>.b.md`
   - sidecar actions: replace `.md` with `.actions.<same nnn>.md`
5. Run `@_plugin/draft-reviewers/correctness/correctness-a-cached` and `@_plugin/draft-reviewers/correctness/correctness-b-cached` independently with identical artifact inputs and separate sidecar `cache_path`/`actions_path` values.
6. Validate both outputs: `# REVIEW`, `Decision: PASS | ADVISORY | BLOCKING`, and `Domains: COR`. Treat `IDs:` as routing data only.
7. Read sidecar actions first.
8. Read sidecar state only when actions are malformed, truncated, contradictory, or insufficient to adjudicate evidence.
9. Merge only evidence-backed COR findings in fidelity, action, template, diff-header, and plugin-constraint scope.
10. Keep concrete single-leg findings when in scope.
11. Drop out-of-domain, unsupported, duplicate, or non-actionable findings.
12. Choose minimal fixes.
13. Write `state_path` as the canonical cache.
14. Write canonical `actions_path` with current OPEN findings only.
15. Emit only the pointer review block.

# Output
```text
# REVIEW
Agent: _plugin/draft-reviewers/correctness
Decision: PASS | ADVISORY | BLOCKING
Domains: COR
IDs: COR-001, COR-002, ...
```
- PASS keeps `Agent:`, `Decision: PASS`, and `Domains: COR`; omit `IDs`.

Return ONLY the fenced `text` block above.

# Constraints
- Do not recursively call an adjudicator.
- Preserve the canonical plugin correctness pointer/actions/cache contract.
