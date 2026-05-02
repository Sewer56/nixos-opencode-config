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

### WOPT-005 — Reviewer Response / Cache Split

- Applies To: review loops where runner needs compact decisions but re-review needs detailed evidence.
- Trigger Signals: output bloat, cache/delta failure, duplicate reasoning.
- Refactor Move:
  - Reviewer final response MUST contain only orchestration data: decision, finding IDs, optional one-line summaries, verified ids, and cache path.
  - Cache file MUST contain detailed finding text, status, evidence, prior decision, and expected fix condition.
  - Runner MUST use final response for routing/parsing.
  - Runner MUST read cache only when it needs detailed findings for fix application, reporting, or final audit.
  - Re-review MUST receive `cache_path` and use cache for detailed evidence.
  - Do not duplicate full finding prose in both final response and cache.
- Quality Guard:
  - Response schema MUST stay stable and parseable.
  - Every ID in response MUST exist in cache.
  - Cache evidence MUST be actionable without rereading unchanged inputs.
  - Do not split response/cache if downstream consumer cannot read the cache file.
- Related Design Patterns: OPT-004, OPT-005, OPT-006.
- Expected Gain: smaller runner context and less duplicated response/cache prose.

```text
Reviewer response:
Decision: BLOCKING
Findings: [F1 one-line summary]
Verified: [STEP-002]
Cache: <path>

Cache file:
F1 full evidence, expected fix condition, prior decision, detailed text

Runner reads response; re-review reads cache.
```
