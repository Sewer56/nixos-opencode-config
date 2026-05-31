# Iterate Workflow Playbook

Reference for direct OpenCode agent/command prompt edits. Shared pattern memory lives in:

- `config/doc/workflow/design-patterns.md` — approved reusable creation/refinement patterns
- `config/doc/workflow/optimize-patterns.md` — approved existing-workflow optimization tactics
- `config/doc/workflow/unproven-patterns.md` — intake for genuinely unproven reusable ideas

`_iterate/edit` treats this file as the iterate playbook, not the shared pattern catalog.

## Pipeline

1. `/iterate/edit` — directly edit agent/command target files, write `PROMPT-ITERATE-EDIT-<slug>.md` plus pattern contract, and run compact reviewers.
2. Pattern selector: `_iterate/edit-pattern-selector` writes `PROMPT-ITERATE-EDIT-<slug>.patterns.md`.
3. Reviewers live in `_iterate/edit-reviewers/`:
   - `integrity` — OpenCode schema, permissions, wiring, scope, self-iteration, optimizer architecture.
   - `pattern-compliance` — generated-edit compliance with selected carry-ins, quality guards, apply-to paths, and validation bullets.
   - `instruction-quality` — LLM instruction clarity, dedup, tight inputs, output schemas, topology economy.

No draft/finalize commands. No handoff or STEP artifacts.

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
- pattern contract = `<artifact_base>.patterns.md`
- reviewer caches:
   - `<artifact_base>.review-integrity.md`
   - `<artifact_base>.review-pattern-compliance.md`
   - `<artifact_base>.review-instruction-quality.md`
- no draft context, finalize handoff, or STEP files

### Reviewer Merge

The old wording/style/clarity/dedup/performance/correctness/diff/meta reviewer spread is collapsed into two merged review domains plus one pattern audit:

- `integrity` keeps high-risk checks separate: command/agent schema, permissions, task refs, self-iteration enforcement, optimize architecture, and scope.
- `pattern-compliance` independently checks generated prompt edits against selected OPT/WOPT carry-ins and guards.
- `instruction-quality` owns overlapping prompt-economy checks: wording, style, clarity, dedup, tight inputs, output schemas, markdown fences, and reviewer topology.

Keep high-risk correctness/security/data-loss checks separate from wording/polish checks. Merge reviewers when they read the same artifacts and emit overlapping findings.

### Pipeline Decomposition

When a monolithic agent prompt bundles phases that don't require the full global context — such as repo search, precondition validation, path resolution, slug derivation, external lookups, or broad discovery — split those phases into standalone pipeline stages:

1. Identify phases that can run with narrow inputs and produce a compact output file.
2. Create a prep agent for each such phase that writes a pipeline state file.
3. Update the downstream agent to read the state file first and fast-fail if missing.
4. Make the prep agent a separate user-facing command when it is a prerequisite gate.
5. Pipeline state file is the single handoff between stages — each stage's prompt contains only its phase.

Example: repo search, slug derivation, and precondition validation can all run before the main agent needs to generate artifacts or run review loops. Lift them into a prep command, have it write a state file, and have the main agent read that file.

See OPT-017 for full carry-in rules and quality guards.

## Direct Edit Loop

- coordination file: `PROMPT-ITERATE-EDIT-<slug>.md`
- pattern contract file: `PROMPT-ITERATE-EDIT-<slug>.patterns.md`
- reviewer caches hold detailed findings; final responses stay compact
- integrity runs first when frontmatter, permissions, wiring, self-iteration, optimizer workflow, or command/agent files change
- pattern-compliance runs every run after integrity and is the only reviewer that receives the pattern contract
- instruction-quality runs for prompts, command bodies, output schemas, subagent calls, and reviewer topology changes
- rerun only reviewer domains touched by fixes

## When to Read What

- Read this file for iterate-specific artifact and self-iteration rules.
- Read `config/doc/workflow/design-patterns.md` for shared creation/refinement prompt design patterns.
- Read `config/doc/workflow/optimize-patterns.md` for existing-workflow prompt optimization tactics.
- Read `config/doc/workflow/optimize-maintenance.md` only when editing `_workflow/optimize*` or `export-analyzer.md`.
- Read `config/doc/workflow/unproven-patterns.md` only for `IDEA-###` intake or promotion.
- Do not read `opencode-source/`; direct prompt edits use local command/agent conventions and workflow docs.
