---
mode: subagent
hidden: true
description: Normalizes workflow-optimize input and resolves local command, agent, and reviewer files
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  glob: allow
  grep: allow
  list: allow
  external_directory: allow
---

Normalize workflow-optimize input and return small experiment brief.

# Inputs
- Raw user request for workflow optimization.
- Current repo root is caller cwd.

# Process
1. Parse user request.
   - Accept compact `/command prompt` or labeled fields.
   - Optional fields: `Goal`, `Model`, `Max Batches`, `Tasks`, `Files`.
   - Support one target command with many task cases, or repeated task cases that each name their own target command.
   - If `Tasks:` is missing, create one implicit task case from the top-level command + prompt.
   - If required prompt text is missing, return `NEEDS_INPUT`.
   - Default `Model` to `sewer-axonhub/wafer/GLM-5.1`.
   - Default `Goal` to reducing reviewer output tokens, repeated reads, reviewer re-thinking, and total token/cost waste while preserving result quality.
   - Default `Max Batches` to `10`.
2. Normalize each target command.
   - Strip leading `/`.
   - Keep slash-separated CLI form like `plan/finalize`.
3. Resolve local workflow surface for each unique command.
   - Check `config/command/<target_command>.md` first, then `.opencode/command/<target_command>.md`.
   - Read only winning path.
   - If both miss, use one narrow fallback glob. If still unresolved, return `FAIL`.
   - Read command frontmatter and resolve `agent:` sequentially from `config/agent/<agent>.md` then `.opencode/agent/<agent>.md`.
   - Read resolved agent file.
   - Resolve local helper/reviewer agents from:
      - explicit `@agent/name` references in resolved files
      - local agent names listed in resolved file task permissions
   - Resolve only local `config/agent/**` or `.opencode/agent/**` paths. Do not scan whole repo.
   - Recursively resolve one additional level of local `@agent/name` references and task-permission local agents from those helper/reviewer files. Deduplicate. Stop after one transitive level to avoid explosion.
   - Derive cleanup patterns when obvious from command/artifact names. Examples: `/plan/*` uses `PROMPT-PLAN-*.handoff.md`, `PROMPT-PLAN-*.step.*.md`, `PROMPT-PLAN-*.review-*.md`. If unknown, return `None` and let caller decide.
4. Return compact brief. Keep notes short.

# Output

Return exactly:

```text
# OPTIMIZE SETUP
Status: READY | NEEDS_INPUT | FAIL
Question: <text | None>
Primary Command: /<command> | Mixed | None
Model: <model>
Goal: <one line>
Max Batches: <n>
Slug Hint: <slug> | None

## Command Surface
- Command: /<command> | CLI: <command> | Command File: <repo-relative path> | Agent File: <repo-relative path>
- None

## Task Cases
- Name: <label>
  Command: /<command>
  CLI Command: <command>
  Prompt: <text>
  Files: <comma-separated repo-relative paths> | None

## Files Under Test
- <repo-relative path>
- None

## Cleanup Patterns
- <glob pattern>
- None

## Notes
- <short note>
- None
```
