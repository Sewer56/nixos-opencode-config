---
mode: subagent
hidden: true
description: Final-gate cacheless topology audit for direct prompt edits
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
<reviewer_contract id="topology-cacheless" mode="cacheless">
Goal: Final cacheless topology audit. Ignore caches; return current findings inline.
Cached counterpart/final gate pair share `topology-body.txt`. Static script owns render/import checks.
</reviewer_contract>

<input_contract>
- `log_path`: absolute edit log.
- `changed_paths`: repo-relative files changed.
- `target_summary`: one-line goal.
- `risk_flags`: compact flags.
- `static_check_path`: optional static result.
</input_contract>

{{
  file="./.opencode/agent/_iterate/edit-reviewers/_templates/topology-body.txt"
  mode=cacheless
}}
