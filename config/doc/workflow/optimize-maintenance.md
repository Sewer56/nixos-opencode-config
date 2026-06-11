# Workflow Optimize Maintenance

Context for future edits to `_workflow/optimize*`, `export-analyzer.md`, and workflow optimization docs. Keep docs outside executable `agent/` and `command/` trees unless a prompt must import them.

## Architecture

- `optimize.md`: runner; setup, samples, export/digest, quality gate, compare, keep/revert, doc memory update.
- `optimize-setup.md`: normalizes request and resolves command/agent/reviewer files plus workflow shape.
- `export-analyzer.md`: profiles one exported session for observable waste signals.
- `{{path:./tools/workflow-optimize/run_batch.py}}`: creates isolated workspaces and runs samples.
- `{{path:./tools/workflow-optimize/export_digest.py}}`: primary evidence for output+reasoning tokens across root and children.

## Goal

Reduce generated tokens without quality loss. For primary+reviewer workflows, runner owns orchestration, artifacts, delta/cache coordination, reviewer fanout, and final gate. Reviewers own domain checks, evidence, cache/action files, and response protocol. Runner passes paths, deltas, trigger flags, and user notes; it does not restate reviewer contracts.

## Prompt Optimization Baseline

Every optimizer-generated prompt edit must apply `prompt-engineering.md` and separate prompt text from harness/config responsibilities. Optimization should remove prompt bloat, duplicated callee rules, broad thoroughness, stale model hacks, prompt/harness mixing, and one-consumer fragments.

## Workflow Shapes

- `primary+reviewers`: main runner plus review subagents and review loop.
- `primary+helpers`: runner plus helper subagents, no review loop.
- `single-agent`: no local helper/reviewer subagents.
- `nested-run`: launches nested `opencode run`, session exports, or harness tools.
- `mixed`: multiple shapes.

## Signal-First Analysis

Analyzer starts from observable signals, then maps strong hypotheses to `WOPT-###`, `OPT-###`, `PE-###`, or `LOCAL:[[name]]`. Do not ask analyzers to hunt for tactic names first.

Signals: generated hotspot, high child spread, tight input violation, overbroad handoff, duplicate reads, duplicate reasoning, scope leakage, review-loop churn, cache/delta failure, output bloat, topology mismatch, model/risk mismatch, prompt/harness mixing, placeholder/XML ambiguity.

## Quality Gate

Keep/revert decisions require baseline, changed workflow, representative samples, token deltas, and lost-finding check. Never accept token savings that remove required blocking coverage, parseable outputs, source boundaries, or verification.
