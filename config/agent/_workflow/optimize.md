---
mode: primary
description: Runs target command in fresh sessions, exports exact conversations, analyzes workflow waste, and iterates on local workflow files
permission:
  "*": deny
  read:
    "*": allow
  edit:
    "*": allow
  write:
    "*": allow
  bash: allow
  glob: allow
  grep: allow
  list: allow
  question: allow
  external_directory: allow
  task:
    "*": deny
    "codebase-explorer": allow
    "_workflow/optimize-setup": allow
    "_workflow/export-analyzer": allow
---

Run exact workflow optimization experiments against local OpenCode command and agent files.

# Inputs
- User input in compact form (`/target/command prompt text`) or labeled form (`Target Command: ...`, `Target Prompt: ...`).
- User may provide one task or multiple tasks/workflows to benchmark as set.
- Optional fields:
  - `Files:` list of attachments to pass through `opencode run --file`
  - `Model:` model for nested experiment runs
  - `Goal:` optimization target
  - `Max Runs:` experiment cap
  - `Tasks:` list of task cases to run sequentially for same workflow

# Scope
- Optimize workflow files and workflow-helper harness only: `config/**`, `.opencode/**`, and `tools/workflow-optimize/**`.
- Do not edit product code, tests, exported bundles, or benchmark artifacts.
- Use fresh sessions for experiments unless the user explicitly asks to continue an existing one.
- Do not use todo lists for this workflow. Keep state in `experiment_log` only.

# Shared helpers
- Use `@_workflow/optimize-setup` first. It returns normalized task cases and resolved workflow files.
- Use `python3 tools/workflow-optimize/run_opencode.py` for every nested run. Do not pipe raw nested JSON into parent output.
- Use `@_workflow/export-analyzer` for every exported run.

# Artifacts
- `slug`: derive 2–3 word slug from target command + goal
- `experiment_log`: `PROMPT-WORKFLOW-OPTIMIZE-<slug>.md` in current working directory
- `runtime_root`: `.opencode/workflow-optimize/<slug>/`
- `events_dir`: `<runtime_root>/events/` for compact run metadata, not full raw archives
- `run_meta`: `<events_dir>/run-###.meta.json`
- `exports_dir`: `<runtime_root>/exports/`
- `db_path`: active OpenCode SQLite path resolved with `opencode db path`
- `opencode_sessions_bin`: `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/tools/opencode-sessions/target/release/opencode-sessions`
- `task_case_index`: ordered list of task cases for multi-task experiments
- `optimization_catalog`: `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/.opencode/WORKFLOW-OPTIMIZATIONS.md`
- `optimization_candidates`: `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/.opencode/WORKFLOW-OPTIMIZATION-CANDIDATES.md`

# Process

## 1. Build setup brief
- Call `@_workflow/optimize-setup` with raw user request first.
- If setup returns `NEEDS_INPUT`, ask one compact question and stop.
- If setup returns `FAIL`, stop with `FAIL`.
- Use setup result as source of truth for:
  - ordered `task_case_index`
  - normalized per-task CLI command
  - `files_under_test`
  - default `Model`
  - default `Goal`
  - default `Max Runs`
- Re-read workflow files only when preparing edits or checking analyzer findings. Do not rediscover repo each run.
- Resolve `db_path` once with `opencode db path` before first run and reuse it for all exports.

## 2. Maintain shared experiment log
- Create or rewrite `experiment_log` early.
- Keep these sections updated:
  - `## Target`
  - `## Goal`
  - `## Model`
  - `## Database`
  - `## Files Under Test`
  - `## Baseline`
  - `## Quality Gate`
  - `## Runs`
  - `## Discarded Attempts`
  - `## Hypotheses`
  - `## Best Current Strategy`
  - `## Rejected Changes`
  - `## Next Experiments`
- Record only smallest facts needed to resume or compare runs.
- Before first edit, define explicit quality gate and record it in `## Quality Gate`.
- Record `db_path` in `## Database`.
- Record task set in log when multiple task cases exist.
- Quality gate must include at least:
  - required completion signal for target task
  - required deliverable or final-answer property
  - failure signals that auto-reject run
- For validation or failure-path experiments, quality gate may explicitly require correct fast-fail behavior instead of final deliverable files.
- Example valid failure-path gate: missing input should return `FAIL` quickly with correct reason and minimal extra work.
- Before running a failure-path validation, verify assumed missing input is truly absent in current workspace.
- If workspace already contains artifact that would satisfy test precondition, change test input to unique nonexistent path or unique slug before baseline run.
- Add comparison priorities explicitly to `## Quality Gate`:
  - `Quality`: hard gate
  - `Performance`: wall time plus stable workflow-efficiency signals
  - `Cost`: tokens, tool calls, and rediscovery overhead
- Because provider latency can jitter, do not rank runs on time alone. Pair time with stable structure metrics like repeated reads, repeated context restatement, subagent rediscovery, cache misses, and token-heavy repeated reasoning.
- When multiple task cases exist, quality gate must define pass/fail per task and aggregate pass rule for whole set.
- Record each run as `PASS`, `FAIL`, or `UNCLEAR`. Only `PASS` runs can become winner.
- Under each run entry, record at least:
  - `Session ID`
  - `Task Case`
  - `Result`
  - `Export Path`
  - `Elapsed`
  - `Tokens`
  - `Tool Calls`
  - `Quality Gate`
  - `Analyzer Decision`
  - `Best Next Move`
  - `Notes`
- After per-task entries, record run aggregate summary with:
  - task-case pass count
  - aggregate tokens
  - aggregate elapsed
  - worst-case reviewer spread
  - biggest scope-leak finding

## 3. Run exact experiment cycle
- Before each run (including baseline), clean target-command artifacts from workspace:
  - Remove files matching the command's artifact patterns (e.g., for `plan/finalize`: `PROMPT-PLAN-*.handoff.md`, `PROMPT-PLAN-*.step.*.md`, `PROMPT-PLAN-*.review-*.md`).
  - Derive cleanup globs from the target command's artifact naming convention.
  - Record cleanup action in experiment log.
  - If cleanup fails (files locked), warn and continue — do not block the run.
- For each run, execute all task cases sequentially. Never in parallel.
- For each task case, call shared helper:
  - `python3 tools/workflow-optimize/run_opencode.py --run <n> --task <label> --command <cli_command> --title <title> [--model <model>] [--file <path>]... --prompt <prompt> --meta-out <run_meta>`
- `cli_command` must be slashless, eg `plan/finalize`, not `/plan/finalize`.
- Keep same selected model across baseline and candidate runs unless model choice itself is experiment variable.
- Use fresh session titles that include slug + run number + task label when task set has multiple cases.
- The helper exits 0 only when the session completed successfully (`reason: "stop"` in the final `step_finish` event). Any other exit means the session was interrupted, errored, or failed.
- If helper exits non-zero, the run is **discarded** — record in `## Discarded Attempts` (one line: session ID + failure reason). Do not export or analyze. Do not record in `## Runs`. Stop current run, proceed to next.

## 4. Export exact session
- Export exact captured `sessionID` with:
  - `<opencode_sessions_bin> --db "<db_path>" export --out "<exports_dir>" <sessionID>`
- `opencode_sessions_bin` is `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/tools/opencode-sessions/target/release/opencode-sessions`. Always use the full absolute path.
- The tool creates an auto-named subdirectory under `<exports_dir>`. Capture the printed output path.
- Save each export under `<exports_dir>`.
- After export, check `session_status` in the export's `index.json` (`root_session_status` field). If `error` or `abandoned`, the run is **discarded** — move from `## Runs` to `## Discarded Attempts`. Do not analyze.
- Record run number, `sessionID`, and export path in `experiment_log`.
- Never use "latest session" heuristics. Current optimizer session is also active.

## 5. Analyze export bundle
- Before calling analyzer, run:
  - `python3 tools/workflow-optimize/export_digest.py <export_path>`
- `export_path` is the auto-named subdirectory created by `opencode-sessions export` (contains `index.json`, `sessions/`, etc.).
- Pass analyzer only: export path, export digest, goal, target command, and files under test.
- Do not make main iterator read full `README.md` or full `index.json` unless digest is missing or clearly inconsistent.
- Do not skip analyzer because run is validation-only. Analyzer may still return `HOLD`, but export bundle must be reviewed.
- Let analyzer own export read order, deeper file escalation, waste signal detection, and reviewer-spread/scope-leak analysis.

## 6. Form one small hypothesis batch
- Prefer smallest workflow change likely to reduce waste while preserving result quality.
- Prefer changes that improve at least one of: quality reliability, elapsed time, or token/cost efficiency without hurting higher-priority metrics.
- For experiment harness bugs, fix harness first before judging target workflow. Do not attribute harness-caused waste to target command.
- Prefer thin command templates when agent already owns workflow logic. Command markdown becomes command-expanded user content; duplicating agent instructions there wastes tokens and can create conflicting guidance.
- When a finding may generalize beyond the current target, update `optimization_candidates`:
  - append next `CAND-###` entry or update existing matching entry
  - record status (`DRAFT` or `TESTING`), scope guess, source experiment log, and evidence still needed
  - mark `LOCAL_ONLY` when evidence points to workflow-specific value only
- Promote into `optimization_catalog` only when evidence is strong enough for reuse (for example: validated across multiple workflows/task sets, or clearly cross-cutting harness behavior with low downside). When promoted, update candidate status to `ADOPTED`, add `OPT-###` to catalog, and update trait matrix if applicable.
- After a **PASS** run that refutes a prior assumption (e.g., per-file steps looked wasteful but actually reduce reviewer context), record it as a `CAND-###` with a `Counter-Intuition` note explaining why naive expectation was wrong and what evidence shows instead.
- Never modify optimization catalog or candidates during discarded or failed runs.
- Always use full absolute paths when writing to `optimization_catalog` and `optimization_candidates`.
- Good targets:
  - tighter subagent inputs
  - shared ledgers or cache files
  - explicit reuse rules for prior artifacts
  - stronger stop conditions
  - less re-discovery boilerplate
  - better read order / skip rules
  - explicit "reuse prior findings" instructions between reviewer/subagent passes
  - token-budget guidance for repeated checks
  - thinner command templates with behavior moved into agent prompt
- Change one coherent batch per run. Avoid mixing unrelated ideas.

## 7. Apply workflow-only edits
- Edit only local workflow files under `config/**`, `.opencode/**`, and `tools/workflow-optimize/**`.
- `tools/workflow-optimize/**` counts as harness, not product code. It may be edited when harness waste blocks good measurement.
- Prefer prompt, command, reviewer, rule, and helper-harness changes over model changes.
- Keep edits exact, minimal, and machine-oriented.
- If a run clearly regresses quality or increases waste without an offsetting gain, revert that batch and mark it in `## Rejected Changes`.
- Any run with quality gate `FAIL` or `UNCLEAR` counts as regression unless user explicitly approves exploratory degradation.

## 8. Compare runs and keep winner
- Treat result quality and task completion as hard gates.
- Rank runs by:
  1. output quality / task completion
  2. performance
  3. cost
- Record quality gate result for every run.
- Record analyzer decision (`IMPROVE` or `HOLD`) and strongest finding in `experiment_log` for every exported run.
- Candidate winner must match or beat current winner on quality gate before waste comparison matters.
- If quality gate is ambiguous, inspect final assistant output and deliverable snapshots. If still ambiguous, mark `UNCLEAR` and do not promote run.
- Compare at least:
  - session success / failure
  - aggregate task-case pass rate
  - worst-case task result
  - elapsed time
  - elapsed spread across reviewers/subagents when present
  - total input/output/cache tokens
  - tool call count
  - low-value + waste turn count
  - high-cost low-value turns
  - redundant read signals
  - repeated context restatement / repeated reasoning signals
  - subagent rediscovery or repeated repo scans
  - reviewer scope leakage (time or reads spent outside assigned domain)
  - child-session errors / stale-reference fallout
- For reviewer-heavy workflows, also compare:
  - max reviewer duration / median reviewer duration ratio
  - max reviewer token share
  - count of reviewers with no findings but high cost
- Keep best current strategy in `experiment_log`.
- For multi-task experiments, judge candidate by collective results across task set, not single best case.
- A change that helps one task but harms others only wins if aggregate quality/performance/cost improves under stated priorities.
- Define "noticeable improvement" as any change large enough to matter for chosen target workflow, such as:
  - quality gate moves `UNCLEAR`/`FAIL` -> `PASS`
  - meaningful drop in tokens, tool calls, repeated reads, or repeated re-thinking
  - meaningful drop in elapsed time that is supported by stable structure metrics too
- Stop after 2 consecutive runs with no noticeable improvement, or at `10` runs.

## 9. Stop rules
- Stop when any condition holds:
  - best run is clearly better and next hypothesis is weak
  - two consecutive runs produce no noticeable improvement
  - `Max Runs` reached
  - target workflow becomes unstable
  - 3 consecutive runs are discarded (stability problem)

## 10. Discarded attempt handling
- A run is **discarded** when any of:
  - `run_opencode.py` exits non-zero (interrupted, errored, exception)
  - `run_opencode.py` reports status other than `COMPLETED`
  - Export `root_session_status` is `error` or `abandoned`
- Discarded runs go in `## Discarded Attempts` — one line per attempt: `Run N: session=<id> reason=<short reason>`
- Do not export, analyze, or compare discarded runs.
- Do not count discarded runs toward the `Max Runs` cap.
- If 3 consecutive runs are discarded, stop the experiment — something is broken.

# Output

Return exactly:

- Plain text only.
- No markdown fence.
- No bullets outside `Files Changed:` value.
- No `**bold**`, backticks, headings, or text before/after block.

```text
Status: SUCCESS | INCOMPLETE | FAIL
Experiment Log: <absolute path>
Runs: <n>
Target Command: /<command> | Mixed
Best Session ID: <id> | None
Best Export Path: <absolute path> | None
Files Changed: <comma-separated repo-relative paths> | None
Summary: <one-line summary>
```
