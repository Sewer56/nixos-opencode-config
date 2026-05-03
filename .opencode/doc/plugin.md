# Plugin Development Workflow

Reference for the `/plugin/draft` → `/plugin/finalize` → `/plugin/implement` → `/plugin/debug` pipeline.

Shared workflow design patterns live in `config/doc/workflow/design-patterns.md`. Use this doc for plugin-specific pipeline behavior; use the shared catalog for reusable prompt and workflow design patterns.

## Command Pipeline

1. `/plugin/draft` — Write `<artifact_base>.draft.md` describing the plugin, its hooks, and constraints.
2. `/plugin/finalize` — Convert the confirmed plan into reviewed STEP files. Writes `<artifact_base>.handoff.md` (includes manifest) and individual STEP files as `<artifact_base>.step.*.md`. Runs cache-backed plugin reviewers with scoped re-review.
3. `/plugin/implement` — Apply the finalized plan, type-check, then debug-iterate until the plugin loads cleanly.
4. `/plugin/debug` — Inspect an existing plugin's debug flag and log path, run with debug enabled, check the co-located log file for issues.

## Draft Review

The draft agent follows the shared `/plan/draft` shape with plugin-specific agents:
- Stage 1: `correctness` — request fidelity, template structure, diff header paths, plugin constraints.
- Stage 2: `documentation` + `wording` in parallel. `wording` owns token density, bullet atomicity, imperative style, deduplication, and clarity.

Coordination: `<artifact_base>.draft.handoff.md` (Delta + Decisions).
Cache: `<artifact_base>.draft.review-<domain>.md`. Iteration cap: 5.
Re-review runs automatically on the initial write; after user modifications the agent reminds that re-review is available on request.

## Reviewers

The finalize agent follows the shared `/plan/finalize` shape with plugin-specific agents: fast draft precondition, `_plugin/finalize-explorer`, cache-backed initial review, scoped re-review, and final gates.

Initial reviewers:

- `_plugin/finalize-reviewers/audit` — Plan fidelity, structure, completeness, plugin constraints, economy, and dead-code cleanup.
- `_plugin/finalize-reviewers/tests` — Verification coverage, typecheck/debug checks, behavior coverage, redundancy, and parameterization.

Re-reviewers:

- `_plugin/finalize-reviewers/audit-rereview` — Cache-first verification of audit fixes.
- `_plugin/finalize-reviewers/tests-rereview` — Cache-first verification of verification fixes.

Final-gate reviewers:

- `_plugin/finalize-reviewers/placement` — Declaration ordering (entry point first, callers before callees).
- `_plugin/finalize-reviewers/performance` — Hot-hook overhead, unbounded work, sync I/O, logging volume, and concurrency risk.

Audit/tests write detailed findings and diffs to cache files. Placement/performance return inline findings.

## Standalone Log Pattern

Plugins write debug logs to `<plugin-dir>/.logs/<plugin-stem>/debug.log` via `fs.appendFile`/`fs.appendFileSync`, controlled by `process.env.XXX_DEBUG`. Write to the co-located log file only — ~~`client.app.log` for debug output~~.

Example: `config/plugins/caveman.ts` → `config/plugins/.logs/caveman/debug.log`

Log directory creation runs inside the plugin init body, only when the debug flag is set.

## Auto-Loading

Plugins placed in `config/plugins/` are automatically discovered and loaded by OpenCode. No `opencode.json` registration entry is required for local plugins.

## Split STEP Files

The finalize agent writes a single handoff (`<artifact_base>.handoff.md`) with Summary, Revision History, Step Index, Delta, and Review Ledger, plus individual STEP files as `<artifact_base>.step.*.md`. Reviewers read only the STEP files that Delta marks as Changed or New. Implementers read the handoff, then each STEP file in order. Stable numbering: gaps are valid, no renumbering.

## Cache Files

Audit/tests own cache files and use `## Delta` plus changed STEP paths to skip unchanged STEP files on re-review:

- `<artifact_base>.review-audit.md`
- `<artifact_base>.review-tests.md`

The finalize agent maintains a `## Delta` section in `<artifact_base>.handoff.md`. Re-review passes only `cache_path`, changed STEP paths, finding IDs, and a fix ledger unless the cache is missing or scope changed.

## Slug Derivation

Each `/plugin/draft` and `/plugin/finalize` agent derives a 2–3 word slug from the request context. The slug becomes part of the base name:

- `artifact_base` = `PROMPT-PLUGIN-PLAN-<slug>` for both draft and finalize
