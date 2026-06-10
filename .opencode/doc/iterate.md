# Iterate Workflow Playbook

Reference for direct OpenCode agent/command/reviewer prompt edits. Shared pattern memory lives in:

- `config/doc/workflow/design-patterns.md` — approved reusable creation/refinement patterns
- `config/doc/workflow/optimize-patterns.md` — approved existing-workflow optimization tactics
- `config/doc/workflow/unproven-patterns.md` — intake for genuinely unproven reusable ideas

`_iterate/edit` treats this file as the iterate playbook, not the shared pattern catalog.

## Pipeline

1. `/iterate/edit` routes to `_iterate/edit`, the primary edit orchestrator.
2. Prep: `_iterate/edit-prep` resolves targets, artifacts, classification, risks, and required reads; it writes `PROMPT-ITERATE-EDIT-<slug>.prep.md`.
3. Pattern selector: `_iterate/edit-pattern-selector` writes `PROMPT-ITERATE-EDIT-<slug>.patterns.md`.
4. Static check: the primary runs `bash scripts/iterate-static-check.sh <artifact_base>`, which writes `PROMPT-ITERATE-EDIT-<slug>.static-check.md` and the `# STATIC CHECK` verdict.
5. Cached review loop: per-domain cached reviewers (integrity, pattern-compliance, prompt-quality, topology) emit pointer review blocks with canonical cache + actions sidecars. The primary applies actions and re-runs touched domains until zero BLOCKING findings or 5 iterations.
6. Final gate (cacheless): per-domain cacheless auditors (integrity-cacheless, pattern-compliance-cacheless, prompt-quality-cacheless, topology-cacheless) ignore prior caches, read the full artifact, and return current findings inline. If any cacheless auditor returns BLOCKING, the runner re-enters the cached loop and repeats the gate.
7. Semantic reviewer bodies live in `_iterate/edit-reviewers/_templates/<domain>-body.txt`. Each cached reviewer and its cacheless counterpart share the same body, parameterized as `mode=cached` or `mode=cacheless`.

No user-confirmed draft/finalize commands by default. Prep/static state files are machine handoffs, not user approval artifacts. Add a draft/finalize boundary only through dedicated future commands when prompt edits need collaborative design before any target edit.

## Shared Pattern Selection

- `_iterate/edit` calls `@_iterate/edit-pattern-selector`.
- Selector reads `config/doc/workflow/design-patterns.md` and `config/doc/workflow/optimize-patterns.md` every run.
- Selector reads `config/doc/workflow/optimize-maintenance.md` only for `_workflow/optimize*` or `export-analyzer.md` edits.
- Selector reads `config/doc/workflow/unproven-patterns.md` only for `IDEA-###` intake or promotion.
- Selector writes selected carry-ins, quality guards, apply-to paths, and validation bullets to `<artifact_base>.patterns.md`.
- Only `pattern-compliance` receives `pattern_contract_path`; it checks generated edits against the selected contract.
- Use `OPT-###` when selected design pattern describes desired steady-state prompt shape.
- Use `WOPT-###` only when refactoring an existing workflow and focus signals match.
- Carry compact rule fragments into changed prompts. Do not paste whole catalogs.
- Target prompts must contain model-facing operational rules directly. Docs cannot be the only source for behavior.

## Split vs. Don't-Split Rule

Adherence drops with prompt length. Narrow units or scripts beat one fat agent. Apply on every add/split/merge/remove of a subagent, reviewer, script, or pipeline phase.

### Split when

- Runner past 150 lines, or holds 3+ heterogeneous domains.
- Reviewer emits findings across unrelated concerns.
- Pipeline phase has no LLM judgement and outputs `PASS | BLOCKING`.
- Caller duplicates callee scope, role, or schema. Move it.
- **Reviewers need different file scopes.** Split: one needs `changed_paths` only, another reads more (callers, error collection, repo scan). Don't drag one into the other's reads.

### Do not split when

- Child is comparable size to parent *and* parent is already narrow (<3 domains). Redistribution, not win.
- Both units read same artifacts and emit overlapping findings. Merge.
- Handoff contract is longer than the work saved.
- Phase is one or two sentences; parent runs it cheaply.

### Two metrics

1. Total prompt size (parent + children).
2. Concerns per unit (heterogeneous domains a model holds at once).

A split that grows total but drops concerns-per-unit 9→2 is a real win. A split that holds total steady and keeps concerns-per-unit is a net loss. Measure concerns-per-unit, not file length.

### Script over subagent

No LLM judgement + deterministic verdict + invoked once per run = use `scripts/<name>.sh`.

The current static check is `scripts/iterate-static-check.sh` for exactly this reason. A subagent that only relays `bash` exit codes is wasted model context.

### Split-cost guard

Each new unit adds: a permission grant, a file to sync, a handoff contract, a set of edits. If the one-line cost is longer than the work removed, do not split.

### Caveman rule

Every NEW or CHANGED rule card, command body, agent prompt, and reviewer prompt follows the caveman full variant. Existing rules grandfathered. See `.opencode/agent/_iterate/rules/caveman-rule.txt`.

### Validation

After every split, merge, or script-replacement:

- Update the runner's `task:` allowlist or `bash:` grant in lockstep.
- Update this iterate doc and any affected reviewer's `cache_path` / artifact names.
- Verify the parent prompt got *shorter* and the new unit is *narrower*. If neither shrank, revert the split.

The `topology` reviewer enforces this rule during review. New reviewer-spread proposals that violate it will be flagged BLOCKING.

## Iterate-Only Conventions

### Self-Iteration

- If target paths include `.opencode/agent/_iterate/**` or `.opencode/command/iterate/**`, set `self_iteration: true`.
- Classify intent as:
  - `wording-only` — text refinement with no enforcement-logic effect
  - `rule-change` — changes to instructions that govern future `/iterate/edit` output
- For rule-change runs, directly update model-facing instructions in `_iterate` agents/reviewers/commands when future behavior changes.
- Documentation-only updates are outside `/iterate/edit` target scope.

### Artifact Shape

- `artifact_base` = `PROMPT-ITERATE-EDIT-<slug>`
- edit log = `<artifact_base>.md`
- prep state = `<artifact_base>.prep.md`
- pattern contract = `<artifact_base>.patterns.md`
- static check result = `<artifact_base>.static-check.md`
- reviewer caches:
   - `<artifact_base>.review-integrity.md`
   - `<artifact_base>.review-pattern-compliance.md`
   - `<artifact_base>.review-prompt-quality.md`
   - `<artifact_base>.review-topology.md`
- no draft context, finalize handoff, or STEP files in the direct workflow

### Reviewer Topology

The old wording/style/clarity/dedup/performance/correctness/diff/meta reviewer spread is collapsed into a static script plus four semantic domains, each with a cached and a cacheless reviewer:

- `scripts/iterate-static-check.sh` owns deterministic changed-path, import/ref, render, whitespace, and markdown-fence checks before semantic review.
- `integrity` and `integrity-cacheless` keep high-risk checks separate: command/agent schema, permissions, task refs, self-iteration enforcement, optimize architecture, source boundary, and scope.
- `pattern-compliance` and `pattern-compliance-cacheless` independently check generated prompt edits against selected OPT/WOPT carry-ins and guards.
- `prompt-quality` and `prompt-quality-cacheless` own prompt-text concerns: LLM runtime writing, tight inputs, output schemas, wording, clarity, dedup.
- `topology` and `topology-cacheless` own workflow-shape concerns: reviewer topology economy, pipeline decomposition, template feature use.

Per-domain cached reviewers run during the review loop. Each reads its own cache + actions sidecar, emits a pointer review block, and lets the runner carry state across iterations. Per-domain cacheless reviewers run only at the final gate, ignore prior caches, read the full artifact, and emit current findings inline. The two phases share a body template, parameterized as `mode=cached` or `mode=cacheless`. See `.opencode/agent/_iterate/edit-reviewers/_templates/<domain>-body.txt`.

Split only when the child has an independent domain and a smaller scoped input. Keep high-risk correctness/security/data-loss checks separate from wording/polish checks. Merge reviewers that read the same artifacts and emit overlapping findings.

### Pipeline Decomposition

When a monolithic agent prompt bundles phases that don't require the full global context — such as repo search, precondition validation, path resolution, slug derivation, external lookups, or deterministic validation — split those phases into standalone pipeline stages:

1. Identify phases that can run with narrow inputs and produce a compact output file.
2. Decide subagent vs. script using the split-decision rule: subagent when LLM judgement is needed, `scripts/<name>.sh` when the phase is deterministic.
3. Create the unit that writes a pipeline state file or verdict file.
4. Update the downstream agent to read the state file first and fast-fail if missing.
5. Pipeline state file is the single handoff between stages — each stage's prompt contains only its phase.

Examples in this iterate:

- `_iterate/edit-prep` owns repo search, slug derivation, and precondition validation before the main agent edits.
- `scripts/iterate-static-check.sh` owns deterministic validation after edits and before semantic reviewers.
- `_iterate/edit-pattern-selector` owns OPT/WOPT selection when the runner should not read the full pattern catalogs.

See OPT-017 for full carry-in rules and quality guards. See `.opencode/agent/_iterate/rules/split-decision-rule.txt` for the operational rule that gates new splits.

## Direct Edit Loop

- coordination file: `PROMPT-ITERATE-EDIT-<slug>.md`
- prep state file: `PROMPT-ITERATE-EDIT-<slug>.prep.md`
- pattern contract file: `PROMPT-ITERATE-EDIT-<slug>.patterns.md`
- static check result file: `PROMPT-ITERATE-EDIT-<slug>.static-check.md`
- reviewer caches hold detailed findings; final responses stay compact
- static check runs before semantic reviewers and after any blocking-fix edit
- cached loop runs first; integrity runs first when frontmatter, permissions, wiring, self-iteration, optimizer workflow, or command/agent files change; pattern-compliance runs every run after integrity and is the only reviewer that receives the pattern contract; prompt-quality runs for prompts, command bodies, output schemas, and subagent calls; topology runs for runner changes, reviewer topology changes, the static-check script, and pipeline decomposition
- cacheless final gate runs after the cached loop converges with zero BLOCKING findings; cacheless auditors run in parallel and ignore prior caches
- rerun only reviewer domains touched by fixes
- if the cacheless final gate returns BLOCKING, apply fixes, re-enter the cached loop, then repeat the final gate
- every NEW or CHANGED rule card, command body, agent prompt, and reviewer prompt must follow the caveman rule (full variant) at `.opencode/agent/_iterate/rules/caveman-rule.txt`; existing rules grandfathered

## When to Read What

- Read this file for iterate-specific artifact, self-iteration, split-vs-don't-split, and caveman rules.
- Read `config/doc/workflow/design-patterns.md` for shared creation/refinement prompt design patterns.
- Read `config/doc/workflow/optimize-patterns.md` for existing-workflow prompt optimization tactics.
- Read `config/doc/workflow/optimize-maintenance.md` only when editing `_workflow/optimize*` or `export-analyzer.md`.
- Read `config/doc/workflow/unproven-patterns.md` only for `IDEA-###` intake or promotion.
- Do not read `opencode-source/`; direct prompt edits use local command/agent conventions and workflow docs.
