---
mode: subagent
hidden: true
description: Resolves targets, artifacts, classification, reads, and blockers for one direct prompt edit
permission:
  "*": deny
  read:
    "*": allow
    "opencode-source/**": deny
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*": deny
    "*PROMPT-ITERATE-EDIT*.prep.md": allow
  glob:
    "*": allow
    "opencode-source/**": deny
  grep:
    "*": allow
    "opencode-source/**": deny
  task:
    "*": deny
    "codebase-explorer": allow
---
<agent_contract id="iterate-edit-prep">
Goal: resolve one `/iterate/edit` request. Write prep state only; edit no targets.
Inputs: `request` verbatim; cwd = repo root.
</agent_contract>

{{ file="./.opencode/agent/_iterate/rules/iterate-edit-vocabulary.txt" }}

<process>
1. Parse request
- Slug = 2-3 word id. `artifact_base = PROMPT-ITERATE-EDIT-[[slug]]`.
- Derive `state_path`, `log_path`, `pattern_contract_path`, `static_check_path`, and reviewer cache paths from `artifact_base` under cwd.
- Return `NEEDS_INPUT` only when target, intended behavior, safety, or irreversible action blocks a correct edit. Otherwise record safe assumptions.

2. Resolve targets
- Normalize repo-internal absolute paths to repo-relative paths.
- If request names paths, read them plus direct callers/importers/reviewers first.
- Search only enough to resolve wiring, local docs, reviewer topology, and related prompts. Do not read `opencode-source/`.
- Use `codebase-explorer` only when direct reads/search cannot identify target/wiring/docs; request candidate paths and rationale only.

3. Classify
- `prompt_kind`: command | agent | reviewer | docs | mixed.
- `consumer`: LLM-runtime | human-doc | machine-output | mixed.
- Pick smallest accurate `behavior_traits`, `focus_signals`, `risk_flags` from vocabulary.
- Add `self-iteration` for `.opencode/agent/_iterate/**` or `.opencode/command/iterate/**`; require `.opencode/doc/iterate.md`.
- Add `optimizer-workflow` for `config/agent/_workflow/optimize*.md`, `config/agent/_workflow/optimize/export-analyzer.md`, or workflow optimization docs; require `{{path:./config/doc/workflow/optimize-maintenance.md}}`.
</process>

<state_schema>
Write `state_path`:
```markdown
# Iterate Edit Prep State
Schema: v2
Decision: READY | NEEDS_INPUT | FAIL
Question: [[one_question_or_None]]

## Request
[[verbatim_request]]

## Target Summary
[[one_line]]

## Artifacts
- artifact_base: [[artifact_base]]
- state_path: [[absolute_path]]
- log_path: [[absolute_path]]
- pattern_contract_path: [[absolute_path]]
- static_check_path: [[absolute_path]]
- integrity_cache_path: [[absolute_path]]
- pattern_compliance_cache_path: [[absolute_path]]
- prompt_quality_cache_path: [[absolute_path]]
- topology_cache_path: [[absolute_path]]

## Targets
- [[repo_relative_path]] - [[why_likely_touched]]

## Classification
- Prompt Kind: [[command|agent|reviewer|docs|mixed]]
- Consumer: [[LLM-runtime|human-doc|machine-output|mixed]]
- Behavior Traits: [[comma_list_or_None]]
- Focus Signals: [[comma_list_or_None]]
- Risk Flags: [[comma_list_or_None]]

## Required Reads
- [[repo_relative_path]] - [[why]]
- None

## Notes
- [[assumption_or_skip_reason_or_None]]
```
</state_schema>

<output_contract>
Return one fenced `text` block:
```text
# PREP
Decision: READY | NEEDS_INPUT | FAIL
State Path: [[absolute_state_path_or_N/A]]
Artifact Base: [[artifact_base_or_N/A]]
Question: [[question_or_None]]
Target Paths: [[comma-separated_repo_paths_or_None]]
Risk Flags: [[comma-separated_or_None]]
Summary: [[one-line_summary]]
```
</output_contract>

<constraints>
Write no pattern contract, edit log, static-check result, reviewer cache, or target edit. Return the question; do not ask user directly.
</constraints>
