# opencode-sessions

Read local OpenCode SQLite database. No OpenCode API. No IPC.

Requires v2 storage: databases without a `session_v2` table exit with an
error.

Machine-first export format.
Vibecoded. Not great code, but does the job.

## Features

- browse recent conversations in ratatui tree
- search sessions by title, id, directory, project
- export conversation subtree into machine-oriented folders
- write small top-level `index.json` entrypoint
- write export-local `README.md` with LLM consumption order
- write per-session `summary.json`, `messages.jsonl`, `tool_calls.jsonl`
- split large prompts, reasoning, tool inputs, tool outputs into `artifacts/`

## Commands

```bash
# interactive browser
cargo run -- tui

# print session tree
cargo run -- tree

# search tree
cargo run -- tree --search "subagent"

# export newest matching conversation
cargo run -- export "crate boundaries"

# export exact session id
cargo run -- export ses_31b81bd3dffeyB0vjszmKa7fzS

# list discovered db files
cargo run -- dbs
```

## TUI quick export

- `e` — export selected node only
- `E` — export whole root conversation for selected node
- `o` — open last exported folder
- `enter` — expand/collapse tree node
- `/` — search

Default export base dir: `tools/opencode-sessions/exports/`.

Top-level `index.json` also includes `iteration_meta` plus `delta_from_previous` for sibling exports in same benchmark series.

Top-level export files:

- `index.json` — machine entrypoint; read first
- `README.md` — export-specific LLM consumption guide
- `sessions/` — per-session bundles

Per-session files:

- `summary.json` — session totals, prompt pointer, narrative, hotspots, tool rollup, child links
- `session_status` — explicit session result: `completed` / `running` / `abandoned` / `error`
- `summary.json.children[].delegation_result_preview` — quick child-result triage from parent summary
- `summary.json.file_access_rollup` / `error_patterns` / `retry_chains` — repeated file churn and failure clusters
- `summary.json.retry_chains[].recovery_strategy` — what agent did after repeated same-tool failures
- `summary.json.file_transition_rollup` — aggregated per-file write history, later re-reads, later rewrites, explicit supersession chain, survives-to-end flag (omitted while session still running)
- `summary.json.session_deliverables` — final modified files with last-write turn and coarse patch intent
- `summary.json.turn_dependency_edges` — aggregated cross-turn rewrite edges, linking overwritten turns to overwriting turns
- `summary.json.last_activity_ms` / `staleness_ms` — distinguish active `running` sessions from stale ones
- `summary.json.session_narrative` / `pivotal_turns` — fast storyline plus strategic jump points
- `summary.json.hot_messages` — consecutive hot-message spans, not only single messages
- `summary.json.children[].delegation_export_reference_status` / `current_export_path_hint` — detect stale export-path references inside child prompts and point at current bundle root
- `runtime.models` / `runtime.providers` — session-level runtime set
- `messages.jsonl` — compact per-message digests
- `messages.jsonl.message_kind` — `user`, `assistant-text`, `assistant-tool-only`, `assistant-mixed`, or `assistant-reasoning-only`
- `messages.jsonl.model` / `provider` — only present when session mixes runtimes
- `messages.jsonl.activity_summary` — compact summary for tool-only or sparse assistant steps
- `messages.jsonl.reasoning_summary` / `reasoning_themes` — middle layer plus cheap reasoning topic hints before opening full reasoning file
- `messages.jsonl.user_intent` / `user_tags` / `alternative_user_intents` — repeated on user messages for message-only consumers
- `turns.jsonl.user_message_index` / `message_index_end` — full message span for each turn
- `turns.jsonl.read_files` / `modified_files` — normalized unique path samples plus counts per turn
- `turns.jsonl.agent_strategy` / `turn_cost_tier` / `turn_effectiveness` / `recommended_attention` — heuristic phase, cost, value, and skip/read guidance labels for optimization
- `turns.jsonl.total_tokens` / `tokens_per_tool_call` — inline per-turn cost signal without recomputing
- `turns.jsonl.change_stats` / `change_intents` — aggregated patch counts, line churn, and coarse semantic edit classes for turn-level edits
- `turns.jsonl.call_purpose_rollup` / `key_diff_preview` — why tools were used in turn and compact strongest-edit preview
- `turns.jsonl.failure_narrative` / `optimization_hints` — machine-targeted why/fix summaries for weak turns
- `tool_calls.jsonl` — compact per-tool-call digests
- `tool_calls.jsonl.patch_summary` — structured patch counts plus hunk/line churn for `apply_patch`
- `tool_calls.jsonl.patch_intent` — coarse patch class: feature / fix / refactor / config / test / docs
- `tool_calls.jsonl.call_purpose` — coarse per-call intent: context-gather / search / build / verify-change / delegate
- `summary.json.tool_rollup[].*_input_tokens_proxy` — approximate per-tool token share from hosting messages, split evenly across tool parts in same message
- `tool_calls.jsonl.error_type` — heuristic tool failure class
- `artifacts/` — full prompt/reasoning/input/output only when useful
