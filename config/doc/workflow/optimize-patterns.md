# Workflow Optimize Patterns

Approved `/workflow/optimize` tactics for existing workflow prompts/tools. Use after export/digest evidence. Do not use this file as a creation catalog.

Refs:
- `WOPT-###` — proven tactic for optimizing an existing workflow.
- `OPT-###` — approved design pattern from `config/doc/workflow/design-patterns.md`; cite when the target design pattern itself is the edit.
- `LOCAL:<name>` — one-run hypothesis not yet worth shared docs.

## How to Use

1. Start from observed focus signals and counterevidence, not tactic names.
2. Seed `/workflow/optimize` Strategy Matrix from `## Focus Signal Map`.
3. Prefer `WOPT-###` for refactor/analysis tactics on existing workflows.
4. Use `OPT-###` when the design catalog directly describes the desired steady-state prompt shape.
5. Use `LOCAL:<name>` only when no WOPT/OPT fits tightly.
6. Convert selected `Refactor Move` and `Quality Guard` bullets into direct edit instructions. Do not paste whole catalog text into generated artifacts.
7. Treat code blocks as generic concrete refactor shapes. Copy the structure, not placeholder names. Keep MUST/WHEN/Do-not wording unambiguous.

## Focus Signal Map

| Focus Signal | Usually Apply |
| --- | --- |
| generated hotspot | WOPT-003, WOPT-004, WOPT-005 |
| tight input violation | OPT-002, OPT-001 |
| overbroad handoff | OPT-002, OPT-005, OPT-012 |
| duplicate reads | WOPT-003, OPT-012, OPT-014 |
| duplicate reasoning | WOPT-001, WOPT-003, OPT-003, OPT-006 |
| scope leakage | WOPT-003, OPT-002, OPT-012, OPT-014 |
| review-loop churn | WOPT-001, WOPT-002, OPT-003, OPT-011 |
| cache/delta failure | WOPT-001, WOPT-005, OPT-003, OPT-006 |
| output bloat | WOPT-005, OPT-004, OPT-005, OPT-009 |
| topology mismatch | WOPT-003, OPT-011, OPT-012 |
| model/risk mismatch | WOPT-004 |
| prompt/context bloat | OPT-001, OPT-005, OPT-010 |

## Approved Tactics

### WOPT-001 — Structural Withholding

- Applies To: existing review loops with stable artifacts and partial revisions.
- Trigger Signals: review-loop churn, duplicate reads, duplicate reasoning, cache/delta failure.
- Refactor Move:
  - Split first-review and re-review data flow.
  - First review MUST receive full needed context and write grounded cache.
  - Re-review MUST receive `cache_path` plus the smallest reliable invalidation input: changed ids/paths, source revision/fingerprint, decisions, or trigger flags.
  - Caller MUST withhold unchanged artifacts from re-review in any form — bodies, paths, references — when cache can safely cover them.
  - Reviewer MUST use cache for unchanged verified records and reread only invalidated material.
- Quality Guard:
  - Reread changed domains, unresolved/open blocking findings, cache-stale records, and decision/trigger-referenced artifacts.
  - If cache is missing or malformed, fall back to full reviewer or full needed read.
  - Do not apply withholding when stable ids/paths do not exist or invalidation cannot be determined safely.
- Related Design Patterns: OPT-003, OPT-006.
- Expected Gain: makes broad unchanged rereads impossible instead of merely discouraging them.

```text
Re-review call:
cache_path=<path>
changed_ids=[STEP-003]
decisions=[DEC-002]
artifact_paths=[handoff, STEP pattern]

Do not mention STEP-001 or STEP-002 in any form — no bodies, paths, or references.
Reviewer preserves cached PASS unless changed_ids/decisions touch its domain.
```

### WOPT-002 — Review Loop Restructuring

- Applies To: multi-reviewer workflows with ordered dependencies between correctness, presentation, and advisory checks.
- Trigger Signals: review-loop churn, output bloat, topology mismatch.
- Refactor Move:
  - Map reviewer dependencies before changing loop order.
  - Run high-risk correctness/security/data-loss reviewers before presentation/style/polish reviewers.
  - Apply blocking fixes from earlier phases before launching downstream presentation reviewers.
  - After a fix, rerun only reviewer domains touched by that fix.
  - Preserve PASS for unchanged domains when cache/invalidation rules make reuse safe.
  - Record or defer advisory-only findings unless workflow explicitly requires advisory cleanup before completion.
- Quality Guard:
  - Never skip required blocking coverage.
  - Always rerun any domain touched by a blocking fix.
  - If a fix changes scope, paths, risk flags, or step count, recompute reviewer set before continuing.
  - Final gate MUST require zero unresolved blocking findings.
- Related Design Patterns: OPT-003, OPT-009, OPT-011, OPT-012.
- Expected Gain: fewer invalidated reviewer passes and fewer full-loop reruns.

Before:

```text
correctness + security + wording + style + docs rerun after any fix
```

After:

```text
phase 1: correctness + security
phase 2: wording + style + docs only after blocking correctness passes
wording-only fix: rerun phase 2 only
advisory-only: log/defer, no full-loop rerun
```

### WOPT-003 — Reviewer Topology Shaping

- Applies To: existing command→subagent workflows with multiple reviewers or overloaded reviewers.
- Trigger Signals: topology mismatch, duplicate reads, duplicate reasoning, generated hotspot, scope leakage.
- Refactor Move:
  - Inspect actual reviewer inputs, file reads, findings, and generated-token hotspots before changing topology.
  - Merge reviewers when they read the same artifacts and emit overlapping findings or reasoning.
  - Split an overloaded reviewer only when it has clean independent subdomains and each child can receive smaller scoped input.
  - After merge/split, update caller routing, reviewer prompts, output parsing, and review ledger/cache ownership.
  - Keep explicit domain boundaries in each reviewer prompt.
- Quality Guard:
  - Reject splits where each child still rereads full context.
  - Reject merges that blur correctness, security, data-loss, or other high-risk ownership.
  - Preserve all required coverage and blocking criteria.
  - Verify the new topology reduces duplicate reads/reasoning or generated hotspot cost.
- Related Design Patterns: OPT-011, OPT-012, OPT-014.
- Expected Gain: lower child spread, less duplicate reasoning, and clearer reviewer ownership.

Merge when:

```text
wording reviewer and style reviewer read same artifacts
both emit overlapping prose edits
```

Split when:

```text
overloaded correctness reviewer has independent API and test domains
api-correctness receives API paths only
test-correctness receives test paths only

Do not split if both children reread full plan.
```

### WOPT-004 — Risk-Based Reviewer Model Tiering

- Applies To: reviewer sets mixing high-risk semantic checks with low-risk mechanical checks.
- Trigger Signals: model/risk mismatch, generated hotspot, output bloat.
- Refactor Move:
  - Assign model tier by reviewer domain risk, judgment load, and failure cost.
  - Keep correctness, security, data-loss, migration, and high-risk semantic reviewers on strong models.
  - Move narrow mechanical reviewers to lower/default models only when evidence shows the task is low-risk and rule-bound.
  - Record downgrade criteria in the workflow or optimization notes.
  - Keep escalation path back to stronger model when risk flags appear.
- Quality Guard:
  - Never downgrade from token cost alone.
  - Require domain/risk evidence and 3 representative PASS samples showing no lost required findings unless the target workflow defines a different threshold.
  - Do not downgrade reviewers that must judge ambiguous semantics, safety, security, data loss, or user intent.
  - Revert downgrade if later evidence shows missed findings or unstable protocol output.
- Related Design Patterns: OPT-011, OPT-012.
- Expected Gain: lower generated/reasoning tokens on mechanical reviewers without weakening critical review.

```text
High tier:
- correctness
- security
- data-loss

Low/default tier:
- formatting
- naming
- protocol-shape
- dead-link

Downgrade only after 3-sample PASS shows no lost required findings.
```

### WOPT-005 — Reviewer Action / Cache Split

- Applies To: review loops where runner needs current fixes while re-review/adjudication needs detailed evidence and history.
- Trigger Signals: output bloat, cache/delta failure, duplicate reasoning.
- Refactor Move:
  - Cached reviewer/adjudicator final response MUST be pointer-only: decision, `Actions:`, `Cache:`, and current finding IDs.
  - Cached actions path MUST be the stable current `<cache_path without .md>.actions.md`; A/B leg actions use `<base>.a.actions.md` and `<base>.b.actions.md`.
  - Actions file MUST contain only current actionable OPEN findings needed for this loop.
  - Actions file MUST be updated each pass. Cache history is the durable audit trail; numbered action files are debug-only and not the runtime contract.
  - Cache file MUST contain full finding text, status, evidence, prior decisions, verified observations, resolved/deferred items, expected fix conditions, and a pointer to latest actions.
  - Cacheless reviewers MUST NOT read or write cache or actions files. Findings MUST be returned inline in the output block with `## Findings` and `## Notes` sections.
  - Cacheless adjudicators MUST parse A/B findings from each leg's inline `## Findings` section. They MUST NOT read sidecar files or emit `Actions:`/`Cache:` pointers.
  - Runner MUST use final response for routing and read `Actions:` for fix application (cached) or inline `## Findings` (cacheless).
  - Runner MUST treat missing, malformed, truncated, ambiguous, or insufficient `Actions:` (cached) or inline findings (cacheless) as a protocol failure to retry/rerun.
  - Runner MUST treat `Cache:` as reviewer-owned state for re-review/adjudication and ledger references, not current fix input.
  - Re-review MUST receive `cache_path` and use cache for detailed evidence.
  - Do not duplicate full history, verified observations, resolved findings, or merge notes in the response or actions file.
- Quality Guard:
  - Response schema MUST stay stable and parseable.
  - Every ID in cached response MUST exist in actions and cache when IDs are used.
  - Actions evidence MUST be actionable without rereading unchanged inputs; cache ledger MUST point to the action files that hold evidence.
  - Cacheless output MUST include all findings inline — no sidecar file references, no `Cache:` or `Actions:` pointers.
  - Do not split response/cache if downstream consumer cannot read the cache file.
- Related Design Patterns: OPT-004, OPT-005, OPT-006, OPT-016.
- Expected Gain: smaller runner context and less duplicated cache/history prose.

```text
CACHED reviewer response:
Decision: BLOCKING
Actions: <cache-base>.actions.md
Cache: <path>

Actions file:
F1 current problem + smallest fix

Cache file:
Latest Actions: <cache-base>.actions.md
F1 full evidence, expected fix condition, prior decision, verified observations, resolved history

CACHELESS reviewer response:
Decision: BLOCKING
IDs: COR-001, COR-002, ...

## Findings
### [COR-001]
Category: FIDELITY
Severity: BLOCKING
Evidence: <section, [P#], path:line>
Problem: <one line>
Fix: <smallest concrete correction>
~~~
--- a/<path>
+++ b/<path>
 unchanged context
-old
+new
 unchanged context
~~~

## Notes
- <optional>

Runner fixes from actions (cached) or inline findings (cacheless); adjudicator/re-review reads actions first, cache only as needed.
```

### WOPT-006 — Coupled-Loop Header Pairing

- Applies To: primary orchestrators that contain two phases sharing a re-dispatch loop.
- Trigger Signals: loop churn where the runner re-runs the full pipeline after a fix in one phase because the coupling between phases is implicit; cross-references in the prompt that point to "step 3" or "the loop" without naming the boundary.
- Refactor Move:
  - Replace two top-level `## N.` and `## N+1.` headers with one `## N.` header plus `### Na.` and `### Nb.` substeps when one phase re-dispatches the other.
  - Add a one- or two-line preamble to the `## N.` header naming the loop direction and trigger (e.g. "BLOCKING re-dispatches Na and repeats Nb").
  - Update all cross-references in the prompt to the `Nb` form so re-entry points are unambiguous.
  - When a third phase enters the loop, add it as `Nc` and rewrite the preamble, do not promote it to a new top-level step.
  - Re-dispatch caps and counters stay per-loop, not per-substep.
- Quality Guard:
  - The preamble MUST mention both the trigger condition and the re-dispatch direction; a header that just numbers substeps does not satisfy this.
  - Do not pair two phases that do not share a re-dispatch — keep them as separate top-level steps.
  - Do not pair when one phase re-dispatches the other conditionally based on a sub-criterion (e.g. "only when file class X"); pair only when the re-dispatch is structural.
- Related Design Patterns: OPT-019.
- Expected Gain: shorter prompts, fewer missed re-dispatches, no accidental full-pipeline restarts when a single phase needs a retry.

```text
Before (implicit loop):
## 2. Implement once
- Dispatch implementer.

## 3. Diff review loop
- Dispatch reviewer. On BLOCKING: re-dispatch step 2.

## 4. Validator-fixer
- ...

After (paired loop):
## 2. Implement and diff review

Implementer writes code; reviewer validates. BLOCKING re-dispatches 2a and repeats 2b.

### 2a. Implement once
- Dispatch implementer.

### 2b. Diff review loop
- Dispatch reviewer. On BLOCKING: re-dispatch 2a.

## 3. Validate and certify
- ...
```

Bad: `## 2. Implement` and `## 3. Review` as siblings — the reader has to infer the loop from the cross-reference.
