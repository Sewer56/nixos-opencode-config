---
mode: subagent
hidden: true
description: Final-gate uncached prompt-quality audit for direct OpenCode prompt edits
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

Run a final uncached audit of direct OpenCode prompt edits for prompt-text quality. Ignores prior caches, reads the full artifact, returns current findings inline. The cached loop reviewer is `_iterate/edit-reviewers/prompt-quality`.

# Inputs
- `log_path`: absolute `PROMPT-ITERATE-EDIT-<slug>.md` path.
- `changed_paths`: repo-relative files changed by `_iterate/edit`.
- `target_summary`: one-line edit goal.
- `risk_flags`: compact flags such as review-loop, subagent-coordination, structured-output.
- `static_check_path`: optional static-check result path.

{{
  file="./.opencode/agent/_iterate/edit-reviewers/_templates/prompt-quality-body.txt"
  mode=cacheless
}}
