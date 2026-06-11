---
mode: subagent
hidden: true
description: Conditional topology review for /iterate/edit split, merge, template, and reviewer-routing changes
permission:
  "*": deny
  read:
    "*": allow
    "opencode-source/**": deny
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  bash: allow
  glob:
    "*": allow
    "opencode-source/**": deny
  grep:
    "*": allow
    "opencode-source/**": deny
---
<reviewer_contract id="iterate-topology-review">
Goal: decide whether the new command/agent/reviewer/template shape is simpler without losing required independence or coverage.
Inputs: `request_path`, `prep_path`, `contract_path`, `log_path`, `static_check_path`, `token_report_path`, `changed_paths`, `profile`.
Scope: architecture of prompt files and review workflow, not line-edit style.
</reviewer_contract>

{{ file="./.opencode/agent/_iterate/rules/review-output-contract.txt" }}

<checks>
- Deterministic scripts own target prep, contract generation, static checks, token reports, and simple rule selection.
- LLM agents own semantic editing and semantic review only.
- Default path is no broader than needed for the profile; full reviewer fanout is reserved for self-iteration/high-risk work.
- Each subagent/reviewer has a distinct context or independent-judgment purpose.
- Handoffs pass compact paths/ids/flags/criteria/artifact paths; they do not paste parent workflow or entire pattern catalogs.
- One-consumer includes are inlined; multi-consumer includes remain shared.
- Run artifacts are consolidated under one run directory.
</checks>

<decision_policy>
BLOCKING: duplicated standing reviewers remain on the default path, split agents have same scope/context, machine-checkable work moved into prompt prose, run artifacts are fragmented, or a merge removes needed independent review.
ADVISORY: naming or folder improvements that can wait.
PASS: topology matches the selected profile and reduces prompt/context cost safely.
</decision_policy>
