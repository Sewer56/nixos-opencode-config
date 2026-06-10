---
mode: subagent
hidden: true
description: Cached pattern-compliance review for direct prompt edits
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
    "*PROMPT-ITERATE-EDIT*.review-pattern-compliance*.md": allow
---

Check changed prompt files against selected workflow patterns during cached loop. `_iterate/edit-reviewers/pattern-compliance-cacheless` owns final gate.

# Inputs
- `log_path`: absolute `PROMPT-ITERATE-EDIT-<slug>.md` path.
- `pattern_contract_path`: absolute `PROMPT-ITERATE-EDIT-<slug>.patterns.md` path.
- `cache_path`: absolute `PROMPT-ITERATE-EDIT-<slug>.review-pattern-compliance.md` path from caller.
- `actions_path`: absolute `<cache_path without .md>.actions.md` path.
- `changed_paths`: repo-relative files `_iterate/edit` changed.
- `target_summary`: one-line edit goal.
- `risk_flags`: compact flags.
- `static_check_path`: optional static-check result path.

{{
  file="./.opencode/agent/_iterate/edit-reviewers/_templates/pattern-compliance-body.txt"
  mode=cached
}}
