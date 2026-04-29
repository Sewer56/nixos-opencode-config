# Iterate Workflow Playbook

Reference for iterate-specific flow. Shared optimization patterns live in:

- `.opencode/WORKFLOW-OPTIMIZATIONS.md` — approved reusable patterns
- `.opencode/WORKFLOW-OPTIMIZATION-CANDIDATES.md` — ideas still being tested

`_iterate/draft` and `_iterate/finalize` should treat this file as the
pipeline playbook, not the shared optimization catalog.

## Pipeline

1. `/iterate/draft` — write `PROMPT-ITERATE-<slug>.draft.md` plus
   `PROMPT-ITERATE-<slug>.draft.handoff.md`
2. `/iterate/finalize` — convert confirmed draft into
   `PROMPT-ITERATE-<slug>.handoff.md` plus `PROMPT-ITERATE-<slug>.step.*.md`
3. Draft reviewers live in `_iterate/draft-reviewers/`
4. Finalize reviewers live in `_iterate/finalize-reviewers/`

## Shared Optimization Selection

- `_iterate/draft` and `_iterate/finalize` call
  `@_iterate/optimization-selector`.
- Selector reads `.opencode/WORKFLOW-OPTIMIZATIONS.md` and returns only the
  approved patterns matching target behavior traits.
- Generated targets should absorb only the selected rule fragments. Do not
  copy whole catalog text into draft or finalize artifacts.

## Iterate-Only Conventions

### Self-Iteration

- If target paths include `.opencode/agent/_iterate/**` or
  `.opencode/command/iterate/**`, set `self_iteration: true`.
- Classify intent as:
  - `wording-only` — text refinement with no enforcement-logic effect
  - `rule-change` — changes to instructions that govern future `/iterate`
    output
- `rule-change` finalize runs must include at least one STEP that updates
  enforcement logic, not only wording around it.

### Draft Artifact Shape

- `artifact_base` = `PROMPT-ITERATE-<slug>`
- draft context = `<artifact_base>.draft.md`
- draft handoff = `<artifact_base>.draft.handoff.md`
- Draft items use `[P#]` labels with free-form explanation followed by a diff
  block.

### Finalize Artifact Shape

- finalize handoff = `<artifact_base>.handoff.md`
- machine steps = `<artifact_base>.step.*.md`
- no separate `machine.md`
- handoff carries Summary, Revision History, Step Index, Delta, and review
  coordination state

### Artifact Naming

- Use `PROMPT-<PIPELINE>-<slug>` base names.
- Draft phase uses `.draft.` as a dot-separated segment.
- Finalize phase drops the `.draft.` segment.
- Wrong: `.draft-handoff.md`
- Correct: `.draft.handoff.md`

### Diff Conventions

- Use full repo-relative paths in diff headers.
- Use `Lines: ~start-end` locators.
- When one item has multiple hunks, each hunk gets its own `Lines:` label.
- STEP header `Lines:` is a union summary; per-hunk labels are authoritative.
- Keep 2+ context lines around each diff hunk.

### Reviewer Diff Output

- When reviewer can determine exact replacement text, include inline unified
  diff after `Fix:`.
- When fix is conceptual, keep `Fix:` prose only.

## Review Loops

### Draft Loop

- draft reviewers: correctness, wording, style, dedup, clarity
- coordination file: `<artifact_base>.draft.handoff.md`
- reviewers receive only artifact paths; they derive the rest from their own
  prompts plus Delta/cache state
- if user edits draft without asking for re-review, remind that re-review is
  available on request

### Finalize Loop

- finalize reviewers live in `_iterate/finalize-reviewers/`
- handoff owns Delta and decisions
- reviewers reopen only changed, new, unresolved, or decision-referenced
  items

## When to Read What

- Read this file for iterate-only artifact and self-iteration rules.
- Read `.opencode/WORKFLOW-OPTIMIZATIONS.md` for shared optimization
  patterns.
- Read `.opencode/WORKFLOW-OPTIMIZATION-CANDIDATES.md` only when evaluating
  whether a new pattern should stay local or become shared.
