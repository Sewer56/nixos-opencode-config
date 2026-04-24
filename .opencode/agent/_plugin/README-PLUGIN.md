# Plugin Development Workflow

Reference for the `/plugin/draft` → `/plugin/finalize` → `/plugin/implement` → `/plugin/debug` pipeline.

## Command Pipeline

1. `/plugin/draft` — Write `PROMPT-PLUGIN-PLAN.md` describing the plugin, its hooks, and constraints.
2. `/plugin/finalize` — Convert the confirmed plan into a reviewed machine plan. Writes `PROMPT-PLUGIN-PLAN.handoff.md` (includes manifest) and individual REV files as `PROMPT-PLUGIN-PLAN.rev.*.md`. Runs 4 diff-returning reviewers.
3. `/plugin/implement` — Apply the machine plan, type-check, then debug-iterate until the plugin loads cleanly.
4. `/plugin/debug` — Inspect an existing plugin's debug flag and log path, run with debug enabled, check the co-located log file for issues.

## Draft Review

The draft agent runs 5 reviewers in `draft-reviewers/` before presenting
the plan to the user:
- `correctness` — plan template structure, diff header paths, plugin constraints
- `dedup` — human/machine zone overlap, `[P#]` cross-item redundancy
- `wording` — token density, bullet atomicity
- `style` — imperative voice, positive framing
- `clarity` — undefined jargon, opaque references

Coordination: `PROMPT-PLUGIN-PLAN.draft-handoff.md` (Delta + Decisions).
Cache: `PROMPT-PLUGIN-PLAN.draft-review-<domain>.md`. Iteration cap: 5.
Re-review runs automatically on the initial write; after user modifications
the agent reminds that re-review is available on request.

## Reviewers

The finalize agent runs four reviewers in parallel:

- `_plugin/finalize-reviewers/errors` — Error-handling coverage, swallowed errors, standalone log pattern compliance.
- `_plugin/finalize-reviewers/reorder` — Declaration ordering (entry point first, callers before callees).
- `_plugin/finalize-reviewers/documentation` — JSDoc coverage, debug flag docs, log path docs.
- `_plugin/finalize-reviewers/correctness` — Plan fidelity, SDK type correctness, no `client.app.log`, no unnecessary `opencode.json` registration.

Each reviewer returns a `## Diff` section with unified diffs so the implement agent can apply fixes mechanically.

## Standalone Log Pattern

Plugins write debug logs to `<plugin-dir>/.logs/<plugin-stem>/debug.log` via `fs.appendFile`/`fs.appendFileSync`, controlled by `process.env.XXX_DEBUG`. Write to the co-located log file only — ~~`client.app.log` for debug output~~.

Example: `config/plugins/caveman.ts` → `config/plugins/.logs/caveman/debug.log`

Log directory creation runs inside the plugin init body, only when the debug flag is set.

## Auto-Loading

Plugins placed in `config/plugins/` are automatically discovered and loaded by OpenCode. No `opencode.json` registration entry is required for local plugins.

## Split REV Files

The finalize agent writes a single handoff (`PROMPT-PLUGIN-PLAN.handoff.md`)
with Summary, Revision History, REV Index, Delta, and Review Ledger, plus
individual REV files as `PROMPT-PLUGIN-PLAN.rev.*.md`. No separate
`machine.md`. Reviewers read only the REV files that Delta marks as
Changed or New. Implementers read the handoff, then each REV file in
order. Stable numbering: gaps are valid, no renumbering.

## Cache Files

Each reviewer owns a cache file (unchanged from before, but reviewers
now use it to skip reading Unchanged REV files):

- `PROMPT-PLUGIN-PLAN.review-errors.md`
- `PROMPT-PLUGIN-PLAN.review-reorder.md`
- `PROMPT-PLUGIN-PLAN.review-documentation.md`
- `PROMPT-PLUGIN-PLAN.review-correctness.md`

The finalize agent maintains a `## Delta` section in `PROMPT-PLUGIN-PLAN.handoff.md`. Reviewers skip Unchanged items on re-runs.
