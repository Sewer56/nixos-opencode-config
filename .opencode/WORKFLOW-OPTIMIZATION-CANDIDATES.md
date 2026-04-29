# Workflow Optimization Candidates

Staging area for ideas not yet proven enough for `.opencode/WORKFLOW-OPTIMIZATIONS.md`.

## Rules

- Use `CAND-###` ids. Next id = highest existing id + 1. Never reuse numbers.
- Status values:
  - `DRAFT` — idea exists, little evidence
  - `TESTING` — active experiments in progress
  - `ADOPTED` — promoted into approved catalog
  - `LOCAL_ONLY` — useful, but not general enough for shared catalog
  - `REJECTED` — tried and not worth keeping
- Every entry should state scope guess, source evidence, and what would promote or demote it.

## Entry Template

### CAND-### — <name>
- Status: <DRAFT | TESTING | ADOPTED | LOCAL_ONLY | REJECTED>
- Scope Guess: <cross-workflow | iterate-family | finalize-family | workflow-optimize / ... | unknown>
- First Seen In: <experiment log path or ref>
- Problem: <waste or failure pattern>
- Proposed Change: <candidate optimization>
- Why It Might Generalize: <reason>
- Evidence Needed: <what would prove shared vs local>
- Promotion Target: `.opencode/WORKFLOW-OPTIMIZATIONS.md` | keep local
- Notes: <short note>

## Active Candidates

### CAND-001 — Central optimization selector
- Status: TESTING
- Scope Guess: cross-workflow
- First Seen In: shared-catalog refactor around `/workflow/optimize` and `/iterate`
- Problem: shared optimization rules drift across prompts and make main agents longer than needed.
- Proposed Change: use one hidden selector subagent backed by approved catalog to map behavior traits to applicable patterns.
- Why It Might Generalize: draft/finalize workflows across pipelines all do trait → pattern selection.
- Evidence Needed: prove selector stays accurate and lowers duplication across `_iterate` plus at least one other pipeline.
- Promotion Target: `.opencode/WORKFLOW-OPTIMIZATIONS.md`
- Notes: if selector overhead outweighs prompt savings, keep catalog but inline small high-frequency subset.

### CAND-002 — Reviewer-set gating for trivial plans
- Status: TESTING
- Scope Guess: finalize-family
- First Seen In: `PROMPT-WORKFLOW-OPTIMIZE-smoke-test.md` and `/plan/finalize` reviewer-spread work
- Problem: trivial finalize tasks can over-spawn reviewers with little value.
- Proposed Change: derive reviewer set from complexity/risk flags so trivial plans use smaller reviewer set.
- Why It Might Generalize: plugin, plan, and iterate finalize flows all have optional high-cost reviewers.
- Evidence Needed: confirm quality holds while tokens/elapsed drop on at least two finalize pipelines.
- Promotion Target: `.opencode/WORKFLOW-OPTIMIZATIONS.md`
- Notes: approved catalog already has broad `OPT-012`; this candidate decides whether stronger, more explicit gating policy should become shared default.
