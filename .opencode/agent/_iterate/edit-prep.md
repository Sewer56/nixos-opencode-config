---
mode: subagent
hidden: true
description: Prepares direct prompt edits by resolving targets, artifacts, risks, reads
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

Prepare direct `/iterate/edit` run. Resolve request shape, prompt targets, artifacts, classification, required reads. Write one prep state file. Edit no targets.

# Inputs
- `request`: verbatim user request.
- Current working directory: repo root.

# Process

## 1. Parse request
- Derive `slug`: 2–3 word identifier.
- Set `artifact_base = PROMPT-ITERATE-EDIT-<slug>`.
- Set `state_path = <cwd>/<artifact_base>.prep.md`.
- Set `log_path = <cwd>/<artifact_base>.md`.
- Set `pattern_contract_path = <cwd>/<artifact_base>.patterns.md`.
- Set `static_check_path = <cwd>/<artifact_base>.static-check.md`.
- Set reviewer cache paths under `<cwd>/`, never review subdir:
  - `integrity_cache_path = <cwd>/<artifact_base>.review-integrity.md`
  - `pattern_compliance_cache_path = <cwd>/<artifact_base>.review-pattern-compliance.md`
  - `prompt_quality_cache_path = <cwd>/<artifact_base>.review-prompt-quality.md`
  - `topology_cache_path = <cwd>/<artifact_base>.review-topology.md`
- If target paths or intended behavior block safe edit, write `Decision: NEEDS_INPUT` with one `Question:` and stop.

## 2. Resolve paths
- Normalize absolute paths inside repo to repo-relative paths.
- When request names target paths, read them and direct callers/reviewers first.
- Use `glob` and `grep` only for command/agent wiring, local docs, reviewer topology, related prompt paths.
- Use `@codebase-explorer` only when direct reads cannot resolve targets, wiring, permission conventions, docs, topology. Tell it: skip `opencode-source/`.
- Keep `opencode-source/` out of targets and required reads.

## 3. Classify
{{ file="./.opencode/agent/_iterate/rules/iterate-edit-vocabulary.txt" }}
- Set `prompt_kind`: command, agent, reviewer, docs, or mixed.
- Set `consumer`: LLM-runtime, human-doc, machine-output, or mixed.
- Select observed `behavior_traits`, `focus_signals`, `risk_flags` from vocabulary only.
- Set `self-iteration` when paths include `.opencode/agent/_iterate/**` or `.opencode/command/iterate/**`.
- Set `optimizer-workflow` when paths include `config/agent/_workflow/optimize*.md` or `config/agent/_workflow/optimize/export-analyzer.md`.
- Add `.opencode/doc/iterate.md` to required reads for self-iteration rule changes.
- Add `config/doc/workflow/optimize-maintenance.md` to required reads only for `optimizer-workflow`.

## 4. Write state
- Write `state_path` before final response.
- Use exact state shape:

```markdown
# Iterate Edit Prep State
Schema: v1
Decision: READY | NEEDS_INPUT | FAIL
Question: <one concise question | None>

## Request
<verbatim request>

## Target Summary
<one line>

## Artifacts
- artifact_base: <artifact_base>
- state_path: <absolute path>
- log_path: <absolute path>
- pattern_contract_path: <absolute path>
- static_check_path: <absolute path>
- integrity_cache_path: <absolute path>
- pattern_compliance_cache_path: <absolute path>
- prompt_quality_cache_path: <absolute path>
- topology_cache_path: <absolute path>

## Targets
- <repo-relative path> — <why likely touched>

## Classification
- Prompt Kind: <command | agent | reviewer | docs | mixed>
- Consumer: <LLM-runtime | human-doc | machine-output | mixed>
- Behavior Traits: <comma-separated | None>
- Focus Signals: <comma-separated | None>
- Risk Flags: <comma-separated | None>

## Required Reads
- <repo-relative path> — <why>
- None

## Notes
- <short note>
- None
```

# Output

Return exactly one fenced `text` block:

```text
# PREP
Decision: READY | NEEDS_INPUT | FAIL
State Path: <absolute state_path | N/A>
Artifact Base: <artifact_base | N/A>
Question: <question | None>
Target Paths: <comma-separated repo-relative paths | None>
Risk Flags: <comma-separated | None>
Summary: <one-line summary>
```

# Constraints
- Edit no target files.
- Write no pattern contract, edit log, static-check result, reviewer caches.
- Ask no user-facing question directly; return `Decision: NEEDS_INPUT` with one question.
