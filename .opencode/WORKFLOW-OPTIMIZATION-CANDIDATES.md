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

### CAND-003 — Per-File Step Scoping Reduces Reviewer Context
- Status: ADOPTED → OPT-018
- Scope Guess: cross-workflow
- First Seen In: `PROMPT-WORKFLOW-OPTIMIZE-plan-finalize.md` (baseline analysis)
- Problem: Naive intuition says splitting machine plans into many small step files is wasteful (more files to read). Reviewer appeared to "waste" reads by reading each step file individually. But per-file steps let each reviewer read only files relevant to its domain (Delta-guided skip of Unchanged items). A monolithic plan forces every reviewer to hold the entire plan — including steps outside their domain — inflating context and inviting scope leakage.
- Proposed Change: Keep per-file step scoping as default. Do not merge step files to "save reads." The reads are cheaper than the inflated context window of a monolithic plan.
- Why It Might Generalize: Any workflow where subagents review subsets of a plan benefit from file-per-step scoping.
- Evidence Needed: N/A — promoted to OPT-018 based on baseline reviewer-spread evidence.
- Promotion Target: `.opencode/WORKFLOW-OPTIMIZATIONS.md` → OPT-018
- Notes: Counter-intuition. Per-file reads cost more tool calls but save far more context tokens than they spend.
