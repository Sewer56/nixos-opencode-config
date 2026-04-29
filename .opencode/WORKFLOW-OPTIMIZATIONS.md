# Workflow Optimizations

Approved catalog for reusable workflow and prompt optimizations.

Related files:
- Approved patterns: `.opencode/WORKFLOW-OPTIMIZATIONS.md` (this file)
- Unproven ideas: `.opencode/WORKFLOW-OPTIMIZATION-CANDIDATES.md`
- Experiment evidence: `PROMPT-WORKFLOW-OPTIMIZE-*.md`

## How to Use

1. Classify target traits first: review loop, subagent coordination, machine-readable output, diff-based machine artifacts, failure-path validation, convention/artifact change, export/session analysis, or nested-run harnessing.
2. Select only the matching patterns.
3. Carry only the selected behavior into target prompts or reviewers. Do not paste the whole catalog into generated files.
4. Keep scope honest. A pattern may be approved for `cross-workflow`, `iterate-family`, `finalize-family`, `workflow-optimize / export-analysis`, or `workflow-optimize / harness`.
5. New ideas start in `.opencode/WORKFLOW-OPTIMIZATION-CANDIDATES.md`. Promote only after enough evidence.

## Trait Matrix

| Trait | Usually Apply |
| --- | --- |
| command delegates to agent | OPT-001 |
| review loop | OPT-003, OPT-004, OPT-005, OPT-010, OPT-012, OPT-013 |
| subagent coordination | OPT-002, OPT-006, OPT-013 |
| machine-readable final output | OPT-004, OPT-005, OPT-009 |
| diff-based machine artifacts | OPT-008, OPT-009, OPT-010 |
| convention or artifact change | OPT-007 |
| path-only helper sections | OPT-011 |
| failure-path validation | OPT-014 |
| export/session analysis | OPT-015, OPT-016 |
| nested `opencode run --format json` harness | OPT-017 |

## Approved Patterns

### OPT-001 — Thin Command Templates
- Scope: cross-workflow
- Apply When: command mainly forwards user input into an agent.
- Skip When: command must materially transform arguments or inject non-trivial fixed structure.
- Carry-In: keep command markdown minimal; move behavior into agent prompt; avoid duplicating agent instructions in command body.
- Expected Gain: fewer conflicting instructions and lower prompt token load.

### OPT-002 — Tight Subagent Inputs
- Scope: cross-workflow
- Apply When: target spawns subagents or reviewers.
- Skip When: callee truly lacks access to needed context.
- Carry-In: pass only artifact paths, scoped deltas, trigger flags, and user notes; do not restate output format, focus lists, role assignment, or blanket read order already owned by callee.
- Expected Gain: lower token use and less off-scope work.

### OPT-003 — Reviewer Cache + Delta
- Scope: finalize-family
- Apply When: workflow reruns reviewers across iterations or revisions.
- Skip When: single-pass flow with no review loop.
- Carry-In: caller maintains `## Delta`; reviewers own cache files; reviewers reread only Changed, New, unresolved, or decision-referenced items; preserve Unchanged verified cache entries.
- Expected Gain: fewer repeated reads and cheaper re-review.

### OPT-004 — Fixed Structured Output Blocks
- Scope: cross-workflow
- Apply When: machine-readable final answers or reviewer outputs matter.
- Skip When: output is intentionally free-form human prose.
- Carry-In: use fenced `text` blocks for plain structured outputs; keep format stable and exact.
- Expected Gain: better parser reliability and less format drift.

### OPT-005 — Reference Instead of Requote
- Scope: cross-workflow
- Apply When: multiple artifacts share context or requirements.
- Skip When: target artifact must stand alone and pointer-only wording would make it unusable.
- Carry-In: reference by section name or file path instead of repeating large content already present elsewhere.
- Expected Gain: lower prompt size, less divergence between copies.

### OPT-006 — Shared Ledger / Coordination File
- Scope: cross-workflow
- Apply When: orchestrator or coordinator needs shared state across subagents.
- Skip When: simple one-shot delegation has no cross-agent arbitration or carry-forward state.
- Carry-In: keep coordination state in one shared file or ledger rather than scattering it across subagent outputs.
- Expected Gain: cleaner arbitration and less rediscovery.

### OPT-007 — Concise Human Docs for Convention Changes
- Scope: cross-workflow
- Apply When: workflow changes conventions, artifacts, or operator-visible behavior.
- Skip When: pure internal refactor with no user-facing effect.
- Carry-In: add short human-facing documentation update; do not turn docs into a second system prompt.
- Expected Gain: change discoverability without large prompt bloat.

### OPT-008 — Diff Line Locators
- Scope: iterate-family
- Apply When: machine artifacts tell implementers or reviewers where to edit.
- Skip When: artifact uses create-only full-file outputs.
- Carry-In: use `Lines: ~start-end` locators, per-hunk labels, and 2+ context lines; context is authoritative.
- Expected Gain: faster targeted reads and fewer locator ambiguities.

### OPT-009 — Nested Code Fence Safety
- Scope: cross-workflow
- Apply When: generated docs or prompts nest fenced code blocks.
- Skip When: no nested fences exist.
- Carry-In: outer fence uses more backticks than inner fence.
- Expected Gain: prevents malformed markdown and accidental fence closure.

### OPT-010 — Reviewer Inline Diffs When Exact
- Scope: finalize-family
- Apply When: reviewer can specify concrete fix text.
- Skip When: finding is conceptual and exact patch is not reliable.
- Carry-In: inline unified diff after `Fix:` when exact; otherwise keep conceptual `Fix:` prose only.
- Expected Gain: easier mechanical application of reviewer feedback.

### OPT-011 — Inline Path Variables
- Scope: iterate-family
- Apply When: a would-be section contains only variable-to-path mappings.
- Skip When: paths need fuller explanation or cross-file policy.
- Carry-In: place path-variable definitions at start of nearest Process/Workflow section instead of creating a dedicated section.
- Expected Gain: shorter prompts and flatter document shape.

### OPT-012 — Triggered Reviewer Sets
- Scope: finalize-family
- Apply When: reviewer cost varies a lot with task complexity or risk.
- Skip When: every reviewer is always required for correctness.
- Carry-In: derive reviewer set from step count, action mix, risk flags, or performance sensitivity; trivial plans collapse to smaller reviewer set.
- Expected Gain: less reviewer fan-out and better elapsed/token profile.

### OPT-013 — Explicit Reviewer Scope Boundaries
- Scope: cross-workflow
- Apply When: multiple reviewers own different domains.
- Skip When: single reviewer owns full judgment.
- Carry-In: define each reviewer domain explicitly; if concern belongs elsewhere, note once without deep investigation; compare reviewer spread and scope leakage in evaluation.
- Expected Gain: less overlap, less token waste, clearer ownership.

### OPT-014 — Fast-Fail Preconditions
- Scope: cross-workflow
- Apply When: missing prerequisite should stop work immediately.
- Skip When: target can recover cheaply from missing inputs.
- Carry-In: run exact minimal precondition check first; if it fails, emit final failure template immediately; do not continue discovery, reviewer calls, or artifact writes.
- Expected Gain: lower failure-path cost and better correctness.

### OPT-015 — Export Digest First
- Scope: workflow-optimize / export-analysis
- Apply When: analyzing `opencode-sessions` export bundles.
- Skip When: export digest is missing or clearly inconsistent.
- Carry-In: start from compact digest with root pointers and child summary paths; avoid reading full `README.md` and full `index.json` by default.
- Expected Gain: major token reduction on large export bundles.

### OPT-016 — Summary-First Escalation
- Scope: workflow-optimize / export-analysis
- Apply When: root `summary.json` already exposes most cost/value signals.
- Skip When: summary layer lacks needed chronology or wording evidence.
- Carry-In: read root `summary.json` first, then `turns.compact.jsonl`, then `messages.compact.jsonl`, then child summaries, then deeper files only if still needed.
- Expected Gain: cheaper session analysis with controlled escalation.

### OPT-017 — Compact Nested-Run Capture
- Scope: workflow-optimize / harness
- Apply When: parent workflow launches nested `opencode run --format json` sessions.
- Skip When: raw full event stream is explicitly requested for debugging.
- Carry-In: use wrapper/helper that streams JSON lines, captures first top-level `sessionID`, writes compact metadata, and prints one short completion line.
- Expected Gain: prevents bulky parent-session archives and preserves exact session identity.
