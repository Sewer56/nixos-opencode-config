---
mode: subagent
hidden: true
description: Cached integrity review for direct prompt edits
model: sewer-axonhub/minimax-m3 # HIGH-INSTRUCTION
variant: medium
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
    "*PROMPT-ITERATE-EDIT*.review-integrity*.md": allow
---

Review cached direct OpenCode prompt edits for integrity. Static script owns render/import checks. `_iterate/edit-reviewers/integrity-cacheless` owns final gate.

# Inputs
- `log_path`: absolute `PROMPT-ITERATE-EDIT-<slug>.md` path.
- `cache_path`: absolute `PROMPT-ITERATE-EDIT-<slug>.review-integrity.md` path from caller.
- `actions_path`: absolute `<cache_path without .md>.actions.md` path.
- `changed_paths`: repo-relative files `_iterate/edit` changed.
- `target_summary`: one-line edit goal.
- `risk_flags`: compact flags, e.g. command-agent, permission, self-iteration, optimizer-workflow, reviewer-topology, json-config.
- `static_check_path`: optional static-check result path.

{{
  file="./.opencode/agent/_iterate/edit-reviewers/_templates/integrity-body.txt"
  mode=cached
}}
