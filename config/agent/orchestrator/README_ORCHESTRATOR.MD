# Orchestrator Overview

This folder defines the multi-agent workflow that turns task descriptions into a machine prompt pack, then into working code, with planning, review, verification, and requirements validation.

## Entry Points
- `plan/split` scans the repo, then creates the human review drafts: `PROMPT-SPLIT.md` and `PROMPT-DRAFT-*.md`.
- `orchestrator/prompt-pack` turns task description files into `PROMPT-*.md`, `PROMPT-PRD-REQUIREMENTS.md`, and `PROMPT-ORCHESTRATOR.md`.
- `orchestrator` is the scheduler that executes an existing `PROMPT-ORCHESTRATOR.md` sequentially.
- `orchestrator/runner/runner` can be used directly for a single prompt when you already have a prompt file.

## Core Artifacts
- `PROMPT-SPLIT.md`: high-level split summary for human review.
- `PROMPT-DRAFT-*.md`: human-friendly draft tasks reviewed before prompt-pack generation.
- `PROMPT-ORCHESTRATOR.md`: ordered prompt list with objectives, dependencies, tests, and requirement coverage.
- `PROMPT-PRD-REQUIREMENTS.md`: requirement inventory with scope and acceptance notes.
- `PROMPT-ORCHESTRATOR.state.md`: resumable state (prompt status, plan paths, base branch).
- `PROMPT-REQUIREMENTS-UNMET.md`: unmet requirement tracking with stage, reason, and evidence.
- `PROMPT-*-PLAN.md` and `PROMPT-*-CODER-NOTES.md`: per-prompt plan and execution notes.
- `PROMPT-ORCHESTRATOR.validation.md`: final requirements validation report written by the requirements checker.

## Short Version
1. Run `plan/split`.
2. Review `PROMPT-SPLIT.md` and `PROMPT-DRAFT-*.md`.
3. Run `orchestrator/prompt-pack`.
4. Run the scheduler.

## End-to-End Flow
- Generate `PROMPT-SPLIT.md` and `PROMPT-DRAFT-*.md` via `plan/split` and get user approval.
- `orchestrator/prompt-pack` takes task descriptions, performs deep discovery, writes the machine prompt pack, and runs requirements preflight.
- The scheduler reads the index, initializes state, detects base branch, and runs prompts in order.
- Each prompt is executed by `@orchestrator/runner/runner` through plan, specialist plan review, implementation, quality gate, and commit.
- After each prompt, the scheduler runs CodeRabbit review when available.
- After all prompts, final requirements validation checks coverage and evidence against `base_branch`.

## Simple Diagrams

### End-to-end pipeline
```
User Input
  |
  v
plan/split
  |  (PROMPT-SPLIT.md, PROMPT-DRAFT-*.md)
  v
Human review + approval
  |
  v
orchestrator/prompt-pack
  |  (PROMPT-*.md, PROMPT-PRD-REQUIREMENTS.md, PROMPT-ORCHESTRATOR.md)
  v
orchestrator (scheduler)
  |
  v
orchestrator/runner/runner (per prompt, sequential)
  |  (code changes + commits)
  v
orchestrator/runner/requirements/requirements-final
  |  (PROMPT-ORCHESTRATOR.validation.md)
  v
Done
```

### Per-prompt runner cycle
```
PROMPT-NN-*.md
  |
  v
Planner -> Plan Review -> Coder -> Quality Gate -> Commit
  |          |               |           |
  |          v               |           v
  |     reviewer feedback    |     reviewer feedback
  v                          v
PROMPT-NN-*-PLAN.md      PROMPT-NN-*-CODER-NOTES.md
```

## Per-Prompt Runner Cycle
- Plan: `@orchestrator/runner/plan/planner` writes a concrete `-PLAN.md` and optional findings.
- Plan review: 5 reviewers run on every plan: correctness (GPT-5), correctness (GLM), economy, test, and performance.
- Implementation: `@orchestrator/runner/code/coder` applies the plan, runs checks, and writes `-CODER-NOTES.md`.
- Quality gate: 3 reviewers run on every change: sanity (GPT-5), sanity (GLM), and test integrity.
- Commit: `@commit` creates repo-style commits, excluding `PROMPT-*` files.

## Rules Model
- Canonical split rules live under `../rules/`: `general.md`, `documentation.md`, `performance.md`, `test-parameterization.md`, `code-placement.md`.
- `../rules/all.md` is the convenience aggregate for `runner/plan/planner`, `runner/plan/plan-correctness-*`, and `runner/plan/plan-performance-reviewer`.
- `runner/code/coder` and the specialist reviewers read the split files they need directly.
- The canonical documentation policy lives in `../rules/documentation.md`.

## Artifacts and Feedback Flow
- **Planner and coder persist artifacts** to disk: `PROMPT-*-PLAN.md` and `PROMPT-*-CODER-NOTES.md`.
- **Runner persists the canonical review ledger** to `PROMPT-*-REVIEW-LEDGER.md`.
- **Reviewers return feedback directly** in their agent response; the runner normalizes it into the ledger.
- **Scheduler state is persisted** in `PROMPT-ORCHESTRATOR.state.md` after each prompt for resume support.
- **Validation output is persisted** by the requirements checker in `PROMPT-ORCHESTRATOR.validation.md` after the final validation run.

## Review Context
- **Plan review**: pass `ledger_path` on re-review so reviewers can read prior decisions and avoid reopening resolved issues.
- **Quality gate**: pass `ledger_path` and `coder_notes_path`; the gate loops with coder only.

## Iterative Updates (Planner and Coder)
- **Planner revisions**: updates the same `PROMPT-*-PLAN.md` across iterations, adding `## Reviewer Concerns (Revision)` and updating `## Plan Notes` and revision history when feedback requests changes.
- **Coder iterations**: appends a new `## Iteration N` block in `PROMPT-*-CODER-NOTES.md` on each run, capturing status, concerns, issues encountered, and issues remaining.

## Roles At A Glance
- **Scheduler**: `orchestrator` manages state, ordering, and final validation; never edits code.
- **Runner**: `orchestrator/runner/runner` coordinates the plan -> build -> gate -> commit cycle per prompt.
- **Planner**: `orchestrator/runner/plan/planner` produces the actionable plan and updates prompt findings/required reads when needed.
- **Coder**: `orchestrator/runner/code/coder` implements changes, runs verification, and records coder notes.
- **Reviewers**: plan reviewers (`plan-correctness-*`, `plan-economy-reviewer`, `plan-test-reviewer`, `plan-performance-reviewer`) and code reviewers (`code-sanity-*`, `code-test-integrity-reviewer`).
- **Validation and Tools**: requirements preflight/final (`orchestrator/runner/requirements/requirements-*`), CodeRabbit (`coderabbit`), and commits (`commit`).

## Why Generated Artifact Sections Exist

These sections are useful after approval for machine execution. They are no longer the first thing a human reviews.

### `PROMPT-*-PLAN.md`
- `## Plan Notes`: captures assumptions and risks for reviewer context.
  - What it improves:
    - Fewer first-pass/multi-pass misunderstandings
    - Faster review alignment
- `### Settled Facts` (`FACT-###`): stores evidence-backed facts with sources.
  - What it improves:
    - Less re-research/tool-call cost
    - Better handoff to cheaper execution models
    - Easier decision traceability
- `## Implementation Steps`: captures the full planned change path in one file, including any required type/error definitions and `use` statements.
  - What it improves:
    - `use` statements prevent missing imports; reducing tool call counts.
    - Single tool call to write entire plan saves costs.
- `## External Symbols`: records reused files/classes and required imports for each touched file.
  - What it improves:
    - Avoids searching for the same files/classes across iterations.
    - Speeds up planner/coder/reviewer handoffs.
- `## Reviewer Concerns (Revision)` + revision history: keeps loop memory across iterations.
  - What it improves:
    - Reviewers don't have to as-heavily think about already addressed issues, only validate.
    - Lowers contradictions when using multiple reviewers; they will treat it as a result of earlier review.
- `## Requirement Trace Matrix`: maps each requirement to implementation refs, test refs, and acceptance criteria.
  - What it improves:
    - Catches missing/partial requirement coverage earlier.
    - Gives reviewers a quick proof map instead of scanning the whole plan.
- `## Revision Impact Table` (revisions): maps changed hunks/steps to affected requirements and tests.
  - What it improves:
    - Makes re-review focus on the real delta.
    - Reduces accidental regressions and re-litigation of unchanged sections.

### `PROMPT-*-CODER-NOTES.md`
- For passing information from Coder to Quality Gate. Not everything the quality gate returns can always be fixed.
- Can also prevent same bug from being 'rediscovered' in later loops.

### `PROMPT-REQUIREMENTS-UNMET.md`
- Sometimes you can't meet all the requirements, so it's important to keep track of that to avoid loops or early exits.

### `PROMPT-ORCHESTRATOR.state.md`
- Prompt status + plan mapping enables safe resume and avoids rerunning completed work, reducing wasted iterations.

### `PROMPT-ORCHESTRATOR.validation.md`
- Final requirement-level verdict ensures "all prompts passed" still maps to PRD coverage, reducing correctness drift.
