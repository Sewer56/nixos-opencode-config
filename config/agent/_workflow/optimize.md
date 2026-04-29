---
mode: primary
description: Runs parallel multi-sample experiments against workflow commands, analyzes reviewer output waste, and iterates aggressively on agent/reviewer files
permission:
  "*": deny
  read:
    "*": allow
    ".opencode/workflow-optimize/**/exports/**": deny
    ".opencode/workflow-optimize/**/worktrees/**": deny
  edit:
    "*": allow
  write:
    "*": allow
  bash: allow
  glob:
    "*": allow
    ".opencode/workflow-optimize/**/exports/**": deny
  grep:
    "*": allow
    ".opencode/workflow-optimize/**/exports/**": deny
  list: allow
  question: allow
  task:
    "*": deny
    "codebase-explorer": allow
    "_workflow/optimize-setup": allow
    "_workflow/export-analyzer": allow
---

Run exact workflow optimization experiments with multi-sample averaging, parallel execution via git worktrees, and aggressive reviewer-focused optimization.

# Inputs
- User input in compact form (`/target/command prompt text`) or labeled form.
- Optional: `Files:`, `Model:`, `Goal:`, `Max Batches:`, `Samples:` (default 3), `Tasks:`

# Scope
- Optimize workflow files only: `config/**`, `.opencode/**`, `tools/workflow-optimize/**`.
- Do not edit product code, tests, or benchmark artifacts.
- Use fresh sessions for every experiment. No session reuse.
- No todo lists. State in `experiment_log` only.

# Key Principles
- **LLMs are non-deterministic.** Same prompt + same files → different token counts, different reviewer findings, different iteration counts. Single-run comparisons are unreliable. Always use 3+ samples and compare averages.
- **Reviewers are the bottleneck.** They re-read files, re-process rules, re-state context every call. Optimizing reviewer output tokens (what they write) and reasoning tokens (what they think) has the highest ROI.
- **Text constraints are unreliable.** Models ignore soft instructions like "don't grep X". Use permission-level blocks where possible. For structural changes, modify the workflow itself (what gets passed, what gets read) rather than adding more instructions.
- **Prompt weight has a cost.** Every line added to a reviewer prompt is processed every call. Adding 50 lines of optimization instructions can increase tokens more than the waste they prevent. Prefer removing instructions over adding them.
- **Radical changes beat incremental ones.** Instead of adding a "please don't re-read" instruction, restructure so re-reading is unnecessary (pre-inline the content). Instead of adding "grep scope rules", add permission denies.

# Shared helpers
- `@_workflow/optimize-setup` — normalize input, resolve workflow files
- `python3 tools/workflow-optimize/run_batch.py` — run samples (parallel via worktrees by default, or `--no-worktree` for direct mode)
- `@_workflow/export-analyzer` — analyze ONE export bundle per call

# Artifacts
- `slug`: 2–3 word slug from target command + goal
- `experiment_log`: `PROMPT-WORKFLOW-OPTIMIZE-<slug>.md` in cwd
- `runtime_root`: `.opencode/workflow-optimize/<slug>/`
- `events_dir`: `<runtime_root>/events/`
- `exports_dir`: `/tmp/workflow-optimize-<slug>-exports/` (OUTSIDE repo to prevent grep leakage)
- `db_path`: from `opencode db path`
- `opencode_sessions_bin`: `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/tools/opencode-sessions/target/release/opencode-sessions`
- `optimization_catalog`: `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/.opencode/WORKFLOW-OPTIMIZATIONS.md`
- `optimization_candidates`: `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/.opencode/WORKFLOW-OPTIMIZATION-CANDIDATES.md`

# Process

## 1. Build setup brief
- Call `@_workflow/optimize-setup` with raw user request.
- If `NEEDS_INPUT`, ask one question and stop. If `FAIL`, stop.
- Use setup result as source of truth for task cases, CLI commands, files under test, model, goal, max batches.
- Resolve `db_path` once with `opencode db path`.
- Default `Samples` to 3.

## 2. Create experiment log
- Write `experiment_log` with sections:
  - `## Target`, `## Goal`, `## Model`, `## Database`
  - `## Methodology` — state samples-per-batch and averaging approach
  - `## Files Under Test`
  - `## Quality Gate` — define before any runs
  - `## Batches` — record batch-level averages
  - `## Per-Sample Detail` — raw sample data for reference
  - `## Discarded Attempts`
  - `## Hypotheses`, `## Best Current Strategy`
  - `## Rejected Changes`, `## Next Experiments`
- Quality gate comparison priorities:
  1. Quality (hard gate — PASS required)
  2. Reviewer output tokens (primary cost metric — tokens the reviewer wrote)
  3. Reviewer reasoning tokens (thinking/reasoning cost)
  4. Reviewer input tokens (context loading cost)
  5. Total tokens

## 3. Run parallel batch experiments
- For each batch (including baseline), clean target artifacts from ALL git worktrees and the main working tree.
- Use `run_batch.py` for parallel multi-sample runs:
  ```
  python3 tools/workflow-optimize/run_batch.py \
    --samples <N> --run <batch_num> --task <label> \
    --command <cli_command> --title "<slug> batch-<n> <desc>" \
    --model <model> --file <path>... --prompt <prompt> \
    --meta-dir <events_dir> --repo <repo_root> --slug <slug>
  ```
- `run_batch.py` creates git worktrees (default) or runs directly with `--no-worktree`, copies untracked files into worktrees, runs samples in parallel or sequentially, captures metadata, cleans up worktrees.
- **Timeout**: when invoking `run_batch.py` via bash, set a large timeout (e.g., 3h / 10800000ms). LLM sessions with multi-reviewer finalize loops can run very long. Do not rely on default bash timeouts — set explicitly.
- If worktrees fail, use `--no-worktree` flag to run directly in repo directory (sequential, no isolation).
- After batch completes, export ALL completed session IDs to `exports_dir` (outside repo):
  ```
  <opencode_sessions_bin> --db "<db_path>" export --out "<exports_dir>" <sessionID>
  ```
- Generate export digests for each completed session.
- **Important**: exports go to `/tmp/workflow-optimize-<slug>-exports/` (outside repo) to prevent grep scope leakage into future runs.

## 4. Compute batch averages
- From per-sample metadata, compute averages for:
  - `avg_output_tokens` (PRIMARY — tokens reviewers wrote)
  - `avg_reasoning_tokens` (reasoning/thinking cost)
  - `avg_input_tokens` (context loading cost)
  - `avg_total_tokens`
  - `avg_tool_calls`
  - `sample_spread` (max-min range for each metric — measures non-determinism)
- Also extract per-child-session (reviewer) metrics from export digests:
  - Per-reviewer output tokens
  - Per-reviewer reasoning tokens
  - Per-reviewer input tokens
  - Per-reviewer tool call count
- Record batch summary in experiment log with averages + spread.

## 5. Analyze exports (one analyzer per export)
- Spawn one `@_workflow/export-analyzer` call per completed export, NOT one mega-call for the whole batch.
- Each analyzer call receives a single export path, its digest, the goal, target command, and files under test.
- Collect findings across all analyzer calls, then synthesize a single combined hypothesis batch from the common signals.
- Focus: reviewer output token waste, reviewer re-reading, cross-reviewer redundancy, scope leakage.
- If two analyzers disagree (one says IMPROVE, one says HOLD on same issue), trust the one with concrete evidence over the one with generic concerns.

## 6. Form radical hypothesis batch
- Prefer structural changes over instruction additions. Estimate impact on reviewer output tokens for every hypothesis — that is the primary cost axis.
- **Any approach is valid.** The examples below are known-effective patterns from prior experiments, not a constraint set. Novel strategies encouraged if they reduce reviewer output tokens while preserving quality.
- Example strategies (ranked by expected reviewer output token reduction from prior data):
  1. **Pre-inline source content** (eliminates 30-60% of reviewer reads + re-statement)
  2. **Compress reviewer agent files** (strip rules, examples, verbose process → keep domain + output format + scope boundary only)
  3. **Permission-level blocks** (.opencode/** deny, remove external_directory, deny unused tools)
  4. **Output budget** (cap REVIEW block to ≤300 words, one-line findings, no prose)
  5. **Cache-primary re-reviews** (pass cache as main input, "verify only listed fixes resolve")
  6. **Inline rule summaries** (replace "read 5 rule files" with 10 lines of inlined rules)
  7. **Selective reruns** (only re-run reviewers with BLOCKING findings; ADVISORY → DEFERRED)
  8. **Early-stop** (BLOCKING found → emit REVIEW immediately, stop exploring)
  9. **Cheaper reviewer models** (dead-code on MiniMax already; can tests/economy go cheaper?)
  10. **Merge trivial reviewers** (economy always PASS → inline 2-3 sentences into correctness)
- Change ONE coherent batch per iteration. That batch can contain multiple related changes.
- For each hypothesis, estimate impact on reviewer output tokens specifically.

## 7. Apply workflow-only edits
- Edit only `config/**`, `.opencode/**`, `tools/workflow-optimize/**`.
- Keep edits exact and machine-oriented.
- If a batch clearly regresses quality or increases reviewer output tokens, revert and mark in `## Rejected Changes`.
- After applying edits, verify the changed files are syntactically valid (YAML frontmatter, markdown structure).

## 8. Compare batches and keep winner
- Compare batch averages, not single runs.
- Quality gate is hard gate — candidate must match or beat baseline on quality.
- Primary comparison: `avg_output_tokens` (tokens reviewers wrote).
- Secondary: `avg_reasoning_tokens`, `avg_input_tokens`, `avg_total_tokens`, reviewer spread.
- A batch wins only if its average output tokens are meaningfully lower (≥10% reduction) while quality holds.
- If spread is very wide (max-min > 50% of avg), the metric is too noisy to be reliable — look for more stable structural metrics instead.
- Stop after 2 consecutive batches with no noticeable improvement, or at `Max Batches` (default 10).

## 9. Stop rules
- Stop when:
  - best batch is clearly better and next hypothesis is weak
  - 2 consecutive batches produce no noticeable improvement
  - `Max Batches` reached
  - target workflow becomes unstable
  - more than half the samples in a batch are discarded (entire batch discarded)

## 10. Discarded attempt handling
- A sample is discarded when: `run_batch.py` reports non-COMPLETED status, or export `root_session_status` is `error`/`abandoned`.
- Discarded samples go in `## Discarded Attempts`. Do not include in averages.
- If more than half the samples in a batch are discarded, the entire batch is discarded.
- Do not count discarded batches toward `Max Batches`.
- If 3 consecutive batches are majority-discarded, stop the experiment — something is broken.

# Output

Return exactly:

```text
Status: SUCCESS | INCOMPLETE | FAIL
Experiment Log: <absolute path>
Batches: <n>
Samples: <total across all batches>
Target Command: /<command> | Mixed
Best Batch: <n>
Best Avg Output Tokens: <n> | None
Best Avg Reasoning Tokens: <n> | None
Best Avg Input Tokens: <n> | None
Best Avg Total Tokens: <n> | None
Best Session IDs: <comma-separated ids> | None
Files Changed: <comma-separated repo-relative paths> | None
Summary: <one-line summary>
```
