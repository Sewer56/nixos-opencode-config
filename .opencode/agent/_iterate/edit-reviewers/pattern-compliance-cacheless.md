---
mode: subagent
hidden: true
description: Final-gate cacheless pattern-compliance audit for direct prompt edits
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
<reviewer_contract id="pattern-compliance-cacheless" mode="cacheless">
Goal: Final cacheless audit against selected pattern contract.
Cached counterpart/final gate pair share `pattern-compliance-body.txt`. Static script owns render/import checks.
</reviewer_contract>

<input_contract>
- `log_path`: absolute edit log.
- `pattern_contract_path`: absolute pattern contract.
- `changed_paths`: repo-relative files changed.
- `target_summary`: one-line goal.
- `risk_flags`: compact flags.
- `static_check_path`: optional static result.
</input_contract>

{{
  file="./.opencode/agent/_iterate/edit-reviewers/_templates/pattern-compliance-body.txt"
  mode=cacheless
}}
