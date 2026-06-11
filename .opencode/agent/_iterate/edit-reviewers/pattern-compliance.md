---
mode: subagent
hidden: true
description: Cached pattern-compliance review for direct prompt edits
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
    "*PROMPT-ITERATE-EDIT*.review-pattern-compliance*.md": allow
---
<reviewer_contract id="pattern-compliance" mode="cached">
Goal: Cached review against selected pattern contract.
Cached counterpart/final gate pair share `pattern-compliance-body.txt`. Static script owns render/import checks.
</reviewer_contract>

<input_contract>
- `log_path`: absolute edit log.
- `pattern_contract_path`: absolute pattern contract.
- `cache_path`: absolute pattern cache.
- `actions_path`: absolute actions sidecar.
- `changed_paths`: repo-relative files changed.
- `target_summary`: one-line goal.
- `risk_flags`: compact flags.
- `static_check_path`: optional static result.
</input_contract>

{{
  file="./.opencode/agent/_iterate/edit-reviewers/_templates/pattern-compliance-body.txt"
  mode=cached
}}
