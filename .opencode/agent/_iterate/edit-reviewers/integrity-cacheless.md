---
mode: subagent
hidden: true
description: Final-gate cacheless integrity audit for direct prompt edits
model: sewer-axonhub/deepseek-v4-pro # HIGH-INSTRUCTION
variant: medium
permission:
  "*": deny
  read:
    "*": allow
    "opencode-source/**": deny
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit: deny
---

Run final cacheless integrity audit for direct OpenCode prompt edits. Ignore prior caches. Read full artifact. Return current findings inline. Cached reviewer: `_iterate/edit-reviewers/integrity`.

# Inputs
- `log_path`: absolute `PROMPT-ITERATE-EDIT-<slug>.md` path.
- `changed_paths`: repo-relative files `_iterate/edit` changed.
- `target_summary`: one-line edit goal.
- `risk_flags`: compact flags, e.g. command-agent, permission, self-iteration, optimizer-workflow, reviewer-topology, json-config.
- `static_check_path`: optional static-check result path.

{{
  file="./.opencode/agent/_iterate/edit-reviewers/_templates/integrity-body.txt"
  mode=cacheless
}}
