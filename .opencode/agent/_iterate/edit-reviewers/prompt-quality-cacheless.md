---
mode: subagent
hidden: true
description: Final-gate cacheless prompt-quality audit for direct prompt edits
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
<reviewer_contract id="prompt-quality-cacheless" mode="cacheless">
Goal: Final cacheless prompt-quality audit. Ignore caches; return current findings inline.
Cached counterpart/final gate pair share `prompt-quality-body.txt`. Static script owns render/import checks.
</reviewer_contract>

<input_contract>
- `log_path`: absolute edit log.
- `changed_paths`: repo-relative files changed.
- `target_summary`: one-line goal.
- `risk_flags`: compact flags.
- `static_check_path`: optional static result.
</input_contract>

{{
  file="./.opencode/agent/_iterate/edit-reviewers/_templates/prompt-quality-body.txt"
  mode=cacheless
}}
