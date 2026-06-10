# Iterate Workflow Playbook

Reference for direct OpenCode LLM-instruction edits. Shared pattern memory:

- `config/doc/workflow/design-patterns.md` — approved prompt design patterns
- `config/doc/workflow/optimize-patterns.md` — approved workflow optimization tactics
- `config/doc/workflow/unproven-patterns.md` — intake for unproven reusable ideas

`_iterate/edit` treats this file as iterate playbook, not pattern catalog.

## Pipeline

1. `/iterate/edit` routes to `_iterate/edit` primary orchestrator.
2. `_iterate/edit-prep` resolves targets, artifacts, classification, risks, required reads; writes `PROMPT-ITERATE-EDIT-<slug>.prep.md`.
3. `_iterate/edit-pattern-selector` writes `PROMPT-ITERATE-EDIT-<slug>.patterns.md`.
4. Primary edits target prompts/docs.
5. `{{path:./scripts/iterate-static-check.py}} <artifact_base>` writes `.static-check.md` and `# STATIC CHECK` verdict.
6. Cached reviewers write cache + actions sidecars; primary applies actions and reruns touched domains until zero BLOCKING or 5 iterations.
7. Cacheless reviewers ignore prior caches, audit full artifact, return inline findings. BLOCKING re-enters cached loop.

No user-confirmed draft/finalize commands by default. Prep/static files are machine handoffs, not approval artifacts.

## Runtime Source Rule

- Command bodies are user messages.
- Agent/reviewer bodies are system prompts.
- Executable prompts carry runtime behavior: role, scope, inputs, process, constraints, output shape, failure behavior, stop/ask conditions.
- Docs explain behavior; docs never replace model-facing prompt rules.

## Shared Pattern Selection

- `_iterate/edit` calls `_iterate/edit-pattern-selector` once.
- Selector reads `design-patterns.md` and `optimize-patterns.md` every run.
- Selector reads `optimize-maintenance.md` only for `_workflow/optimize*` or `export-analyzer.md` edits.
- Selector reads `unproven-patterns.md` only for `IDEA-###` intake/promotion.
- Selector writes compact carry-ins, guards, apply-to paths, validation bullets to `<artifact_base>.patterns.md`.
- Only `pattern-compliance` gets `pattern_contract_path`.
- Use `OPT-###` for desired steady-state prompt shape.
- Use `WOPT-###` for existing-workflow refactor with matching focus signals.
- Carry compact rules into prompts. Keep catalogs out.

## Split Rule

Adherence drops with prompt length. Narrow unit/script beats fat agent. Apply on add/split/merge/remove of subagent, reviewer, script, phase.

### Split when

- Runner exceeds 150 lines or holds 3+ heterogeneous domains.
- Reviewer emits unrelated findings.
- Phase has no LLM judgment and returns `PASS | BLOCKING`.
- Caller repeats callee scope, role, or schema.
- Reviewers need different file scopes.

### Keep when

- Child size matches parent and parent already narrow.
- Units read same artifacts and emit overlapping findings.
- Handoff contract exceeds work saved.
- Phase is one or two sentences.

### Metrics

1. Total prompt size: parent + children.
2. Concerns per unit: heterogeneous domains model holds.

Split wins when concerns drop, even if total grows. Split loses when concerns stay. Measure concerns, not file length.

### Script over subagent

No LLM judgment + deterministic verdict + once per run = script. Current static check is `scripts/iterate-static-check.py` for this reason.

### Validation

- Update runner `task:` allowlist or `bash:` grant.
- Update this doc and reviewer `cache_path` / artifact names.
- Verify parent shorter and child narrower. Else revert.
- Topology reviewer enforces this rule.

## Caveman Rule

Every NEW or CHANGED rule card, command body, agent prompt, reviewer prompt follows caveman rule. Existing unchanged rules stay.

## Self-Iteration

- If target paths include `.opencode/agent/_iterate/**` or `.opencode/command/iterate/**`, set `self-iteration` risk.
- Classify intent:
  - `wording-only` — text refinement, no enforcement effect
  - `rule-change` — instruction changes governing future `/iterate/edit` output
- For rule-change, update model-facing `_iterate` agents/reviewers/commands and docs together.
- Documentation-only update is outside prompt behavior unless user asks docs.

## Artifacts

- `artifact_base` = `PROMPT-ITERATE-EDIT-<slug>`
- edit log = `<artifact_base>.md`
- prep state = `<artifact_base>.prep.md`
- edit log shape rule = `.opencode/agent/_iterate/rules/edit-log-shape.txt`
- pattern contract = `<artifact_base>.patterns.md`
- static check = `<artifact_base>.static-check.md`
- reviewer caches:
  - `<artifact_base>.review-integrity.md`
  - `<artifact_base>.review-pattern-compliance.md`
  - `<artifact_base>.review-prompt-quality.md`
  - `<artifact_base>.review-topology.md`
- cached reviewer actions = `<cache_path without .md>.actions.md`
- no draft context, finalize handoff, STEP files in direct workflow

## Reviewer Topology

- Static script owns deterministic changed paths, imports/refs, render, whitespace, markdown fence checks.
- `integrity` owns command/agent schema, permissions, task refs, self-iteration enforcement, optimize architecture, source boundary, scope.
- `pattern-compliance` owns selected OPT/WOPT carry-ins and guards.
- `prompt-quality` owns runtime prompt writing, tight inputs, output schemas, wording, caveman, clarity, dedup.
- `topology` owns reviewer topology economy, pipeline decomposition, template use.
- Cached reviewers return pointer `# REVIEW` blocks with `Cache:`, `Actions:`, `## Findings`, `## Verified`; current fixes live in actions sidecars.
- Cacheless reviewers return inline findings; no cache/action paths.
- Cached/cacheless reviewer pairs share `_templates/<domain>-body.txt`; cached output uses `_templates/cached-footer.txt`.

## Direct Edit Loop

- Static check runs before semantic review and after each blocking-fix edit.
- Integrity runs first when frontmatter, permissions, wiring, self-iteration, optimizer workflow, or reviewer topology changes.
- Pattern-compliance runs every cached loop after integrity.
- Prompt-quality runs for prompts, command bodies, output schemas, subagent calls.
- Topology runs for runner changes, reviewer topology, static-check script, pipeline decomposition.
- Rerun only reviewer domains touched by fixes.
- Final gate runs after cached loop convergence; auditors run in parallel and ignore caches.
- If final gate returns BLOCKING, apply fixes, re-enter cached loop, repeat gate.

## Reads

- Read this file for iterate artifact, self-iteration, split, caveman, topology rules.
- Read `design-patterns.md` for prompt design patterns.
- Read `optimize-patterns.md` for existing-workflow optimization tactics.
- Read `optimize-maintenance.md` only for `_workflow/optimize*` or `export-analyzer.md`.
- Read `unproven-patterns.md` only for `IDEA-###` intake/promotion.
- Keep `opencode-source/` unread; use local command/agent conventions and workflow docs.
