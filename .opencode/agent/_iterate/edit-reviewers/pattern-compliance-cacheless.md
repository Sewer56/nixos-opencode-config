---
mode: subagent
hidden: true
description: Final-gate cacheless pattern-compliance audit for direct prompt edits
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
  edit: deny
---

Run final cacheless audit of generated prompt edits against selected pattern contract. Ignore prior caches. Read full artifact. Return current findings inline. Cached reviewer: `_iterate/edit-reviewers/pattern-compliance`.

# Inputs
- `log_path`: absolute `PROMPT-ITERATE-EDIT-<slug>.md` path.
- `pattern_contract_path`: absolute `PROMPT-ITERATE-EDIT-<slug>.patterns.md` path.
- `changed_paths`: repo-relative files `_iterate/edit` changed.
- `target_summary`: one-line edit goal.
- `risk_flags`: compact flags.
- `static_check_path`: optional static-check result path.

{{
  file="./.opencode/agent/_iterate/edit-reviewers/_templates/pattern-compliance-body.txt"
  mode=cacheless
}}
