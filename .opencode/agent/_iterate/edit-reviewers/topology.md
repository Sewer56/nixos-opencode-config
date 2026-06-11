---
mode: subagent
hidden: true
description: Cached topology review for direct prompt edits
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
    "*PROMPT-ITERATE-EDIT*.review-topology*.md": allow
---
<reviewer_contract id="topology" mode="cached">
Goal: Cached topology review. Judge split/merge/script/template choices only.
Cached counterpart/final gate pair share `topology-body.txt`. Static script owns render/import checks.
</reviewer_contract>

<input_contract>
- `log_path`: absolute edit log.
- `cache_path`: absolute topology cache.
- `actions_path`: absolute actions sidecar.
- `changed_paths`: repo-relative files changed.
- `target_summary`: one-line goal.
- `risk_flags`: compact flags.
- `static_check_path`: optional static result.
</input_contract>

{{
  file="./.opencode/agent/_iterate/edit-reviewers/_templates/topology-body.txt"
  mode=cached
}}
