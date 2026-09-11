# AGENTS.md

Repo map for `/iterate/edit` sessions. Personal OpenCode configuration repo;
`~/.config/opencode` links to `config/`. `/home/sewer/opencode` resolves here.

## Edit targets

- `config/agent/` — agents and subagent dirs
  (`_implement/`, `_plan/`, `_refactor/`, `_review/`, `_write/`, `subagent/`).
- `config/command/` — commands (`draft`, `implement`, `write/`, ...).
- `config/plugins/` — local plugins; `opencode-plugin-md-expand` drives
  expanded token counts (submodule).
- `config/opencode.json` — main config.
- `.opencode/agent/`, `.opencode/command/` — repo-local agents and commands
  (`_iterate/`, `migrate`).

Shared rules live in `config/rules/`, organized by consumer family.
Workflow-local fragments live in `config/agent/**/shared/*.txt`.

Wording counts in every importer's expanded tokens.

`README.md` lists commands, workflows and outcomes for users; update it when
these change.

## Checks

- `scripts/count-instruction-tokens.py` — raw/expanded token counts;
  Markdown expansion needs Bun.
- `scripts/validate-opencode-config.py` — static validation of config,
  permissions, task graph, imports.
  Exempts the `_iterate/edit` agent body from prompt-structure checks.
- `scripts/check-workflows.sh` — plan-workflow routing smoke test;
  helpers in `config/scripts/`.
- `rust-llm-tidy` — on PATH via Nix; source is the `tools/rust-llm-tidy/`
  submodule.
