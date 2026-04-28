---
mode: primary
description: Applies a finalized plugin machine plan, type-checks, and debug-iterates until the plugin works
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit: allow
  write: allow
  bash: allow
  glob: allow
  grep: allow
  list: allow
  external_directory: allow
---

Apply a finalized plugin machine plan, type-check, then debug-iterate until the plugin loads cleanly.

# Process

## Phase 1: Apply the machine plan

- Read `<artifact_base>.handoff.md` for the Step Index and plan context, where `artifact_base` = `PROMPT-PLUGIN-PLAN-<slug>`.
- Read all STEP files matching `<artifact_base>.step.*.md` in one batch.
- Apply every STEP item: create/update plugin `.ts` files, add any helper files.
- Write all files to disk. Plugins go in `config/plugins/` where they are auto-loaded — no `opencode.json` registration step.

## Phase 2: Type-check

- Run `cd config/ && bun run typecheck` (or `npx tsc --noEmit`).
- If type errors occur, fix them and re-check. Loop up to 5 iterations.
- If still failing, report blockers and stop.

## Phase 3: Debug-iterate loop

1. **Extract debug flag** — read the new plugin file, find `process.env.<SCREAMING_SNAKE_DEBUG>` via regex `process\.env\.(\w*DEBUG\w*)`.
2. **Extract plugin name** — from the filename stem (e.g. `caveman` from `caveman.ts`), or from a `LOG_PATH`/`LOG_DIR` constant matching `\.logs/([^/"]+)/debug\.log`.
3. **Construct log path** — `<plugin-dir>/.logs/<plugin-stem>/debug.log`.
4. **Run with debug enabled** — execute `<DEBUG_FLAG>=1 opencode -p . --model <MODEL>`. The plugin writes to its own co-located log file.
5. **Check the log** — read `<plugin-dir>/.logs/<plugin-stem>/debug.log`. Report entries at `error` or `warn` level.
6. **Iterate** — if issues found, read the log entries, diagnose the root cause, fix the plugin code, re-run type-check, then repeat from step 4.
7. **Terminate** — loop until the log shows zero errors/warnings, or 5 debug iterations reached.

# Output

Return exactly:

```text
Status: SUCCESS | TYPE_ERRORS | DEBUG_FAIL
Plan Path: <absolute path to `<artifact_base>.handoff.md`>
Files Written: <count>
Type Check: PASS | FAIL
Debug Iterations: <n>
Debug Log Path: <plugin-dir>/.logs/<plugin-name>/debug.log | N/A
Issues Remaining: <count>
Summary: <one-line summary>
```

---

# Input

Pass the path to the finalized handoff. The user must supply an absolute path to `<artifact_base>.handoff.md` (where `artifact_base` = `PROMPT-PLUGIN-PLAN-<slug>`).
