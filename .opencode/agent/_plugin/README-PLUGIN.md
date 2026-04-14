# Plugin Development Workflow

Reference for the `/plugin/draft` → `/plugin/finalize` → `/plugin/implement` → `/plugin/debug` pipeline.

## Command Pipeline

1. `/plugin/draft` — Write `PROMPT-PLUGIN-PLAN.md` describing the plugin, its hooks, and constraints.
2. `/plugin/finalize` — Convert the confirmed plan into a reviewed machine plan (`PROMPT-PLUGIN-PLAN.handoff.md` + `PROMPT-PLUGIN-PLAN.machine.md`). Runs 4 diff-returning reviewers.
3. `/plugin/implement` — Apply the machine plan, type-check, then debug-iterate until the plugin loads cleanly.
4. `/plugin/debug` — Inspect an existing plugin's debug flag and log path, run with debug enabled, check the co-located log file for issues.

## Reviewers

The finalize agent runs four reviewers in parallel:

- `_plugin/reviewers/errors` — Error-handling coverage, swallowed errors, standalone log pattern compliance.
- `_plugin/reviewers/reorder` — Declaration ordering (entry point first, callers before callees).
- `_plugin/reviewers/documentation` — JSDoc coverage, debug flag docs, log path docs.
- `_plugin/reviewers/correctness` — Plan fidelity, SDK type correctness, no `client.app.log`, no unnecessary `opencode.json` registration.

Each reviewer returns a `## Diff` section with unified diffs so the implement agent can apply fixes mechanically.

## Standalone Log Pattern

Plugins write debug logs to `<plugin-dir>/.logs/<plugin-stem>/debug.log` via `fs.appendFile`/`fs.appendFileSync`, controlled by `process.env.XXX_DEBUG`. Write to the co-located log file only — ~~`client.app.log` for debug output~~.

Example: `config/plugins/caveman.ts` → `config/plugins/.logs/caveman/debug.log`

Log directory creation runs inside the plugin init body, only when the debug flag is set.

## Auto-Loading

Plugins placed in `config/plugins/` are automatically discovered and loaded by OpenCode. No `opencode.json` registration entry is required for local plugins.

## Cache Files

Each reviewer owns a cache file:

- `PROMPT-PLUGIN-PLAN.review-errors.md`
- `PROMPT-PLUGIN-PLAN.review-reorder.md`
- `PROMPT-PLUGIN-PLAN.review-documentation.md`
- `PROMPT-PLUGIN-PLAN.review-correctness.md`

The finalize agent maintains a `## Delta` section in `PROMPT-PLUGIN-PLAN.handoff.md`. Reviewers skip Unchanged items on re-runs.
