---
mode: subagent
hidden: true
description: Cached prompt-quality review for direct prompt edits
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
  edit:
    "*": deny
    "*PROMPT-ITERATE-EDIT*.review-prompt-quality*.md": allow
---
<reviewer_contract id="prompt-quality" mode="cached">
Goal: Cached prompt-quality review. Judge runtime instruction density, clarity, and evidence gates.
Cached counterpart/final gate pair share `prompt-quality-body.txt`. Static script owns render/import checks.
</reviewer_contract>

<input_contract>
- `log_path`: absolute edit log.
- `cache_path`: absolute prompt-quality cache.
- `actions_path`: absolute actions sidecar.
- `changed_paths`: repo-relative files changed.
- `target_summary`: one-line goal.
- `risk_flags`: compact flags.
- `static_check_path`: optional static result.
</input_contract>

{{
  file="./.opencode/agent/_iterate/edit-reviewers/_templates/prompt-quality-body.txt"
  mode=cached
}}
