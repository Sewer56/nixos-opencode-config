# Workflow Optimize Maintenance

Context for future edits to `optimize.md`, `export-analyzer.md`, and workflow-optimization docs. Documentation only; keep docs outside `agent/` and `command/` trees so markdown is not parsed as agents or commands.

## Architecture

- `optimize.md` — experiment runner: setup, 3-sample batches, export/digest, quality gate, compare, keep/revert, doc memory updates.
- `optimize-setup.md` — normalizes user request, resolves command/agent/reviewer files plus workflow shape.
- `export-analyzer.md` — profiles one exported session at time. Find observable waste signals, not decide winning optimization.
- `run_batch.py` — creates fresh isolated workspaces, runs exactly 3 samples.
- `export_digest.py` — gives primary comparison evidence: tree output+reasoning tokens across root + children.

## Core Goal

Reduce generated tokens (`output + reasoning`) without quality loss. For primary+reviewer targets, optimize command/subagent cooperation:

- Primary runner owns orchestration, artifact lifecycle, delta/cache coordination, reviewer fanout, and final quality gate.
- Review subagents own domain checks, evidence gathering, cache details, and response format.
- Runner should pass paths, deltas, trigger flags, and user notes; avoid restating reviewer-owned contracts.
- Reviewers should read changed/new/unresolved/decision-referenced material, not every artifact by default.

## Workflow Shapes

- `primary+reviewers`: main runner plus review subagents and review loop. Current main focus.
- `primary+helpers`: main runner plus helper subagents but no review loop.
- `single-agent`: no local helper/reviewer subagents.
- `nested-run`: launches nested `opencode run`, session exports, or harness tools.
- `mixed`: multiple shapes across task cases.

## Signal-First Principle

Do not ask analyzers to hunt for named strategies. That anchors them and causes force-fitting. Analyzer should profile observable signals first, then map strong hypotheses to `WOPT-###`, `OPT-###`, or `LOCAL:<name>` refs.

Focus signals:

- generated hotspot / high child spread
- tight subagent input violations
- overbroad handoff
- duplicate reads
- duplicate reasoning across subagents
- scope leakage
- review-loop churn
- cache/delta failure
- output bloat
- topology mismatch
- model/risk mismatch
- prompt/context bloat
- counterevidence and quality risk

## Strategy Sources

`optimize.md` should not carry embedded strategy definitions. It builds Strategy Matrix from:

- `config/doc/workflow/optimize-patterns.md` — approved existing-workflow `WOPT-###` tactics and Focus Signal Map
- `config/doc/workflow/design-patterns.md` — approved reusable design `OPT-###` patterns
- `LOCAL:<name>` hypotheses for target-specific moves

Common refs:

- generated hotspot → WOPT-003, WOPT-004, WOPT-005
- tight subagent input violations → OPT-002, OPT-001
- overbroad handoff → OPT-002, OPT-005, OPT-012
- duplicate reads / scope leakage → WOPT-003, OPT-012, OPT-014
- duplicate reasoning → WOPT-001, WOPT-003, OPT-003, OPT-006
- loop churn → WOPT-001, WOPT-002, OPT-003, OPT-011
- cache/delta failure → WOPT-001, WOPT-005, OPT-003, OPT-006
- output bloat → WOPT-005, OPT-004, OPT-005, OPT-009
- topology/model mismatch → WOPT-003, WOPT-004, OPT-011, OPT-012

## Pattern Memory

Canonical docs live under `config/doc/workflow/`:

- `design-patterns.md` — approved creation/refinement patterns
- `optimize-patterns.md` — approved existing-workflow optimization tactics
- `template-library.md` — catalog of reusable prompt fragments and rule templates
- `unproven-patterns.md` — empty intake for genuinely unproven reusable ideas

If experiment teaches reusable behavior:

- add or update `OPT-###` only for proven creation/refinement design guidance
- add or update `WOPT-###` only for proven existing-workflow optimization tactics
- add `IDEA-###` only when idea is genuinely unproven but likely reusable
- keep local-only wording or one-off cleanup in experiment log
- update Trait Matrix or Focus Signal Map when adding approved refs

Do not paste whole docs into target prompts. Carry only selected behavior into target workflow files.

## Quality Guard

Primary metric: export-derived tree `output+reasoning` tokens. Quality gate comes first. Never save tokens by weakening correctness, security, required coverage, or changed-domain re-review.

Wins can be small if quality holds and average generated tokens drop. Treat noisy/equal results as basically same and prefer simpler prompt/structure.

## Future Iteration Ideas

- Add richer workflow-shape detection in `optimize-setup.md` if setup output becomes unreliable.
- Add digest support for same-reviewer rereads and response/cache duplication if export summaries expose enough detail.
- Keep `WOPT-###` tactics focused on existing-command optimization; move only creation-useful invariants into `OPT-###`.
