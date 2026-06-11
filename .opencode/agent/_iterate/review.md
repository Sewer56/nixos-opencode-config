---
mode: subagent
hidden: true
description: Default semantic prompt review for /iterate/edit contracts
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
<reviewer_contract id="iterate-prompt-review">
Goal: decide whether changed prompt/docs text satisfies the compiled contract without unnecessary prompt cost.
Inputs: `request_path`, `prep_path`, `contract_path`, `log_path`, `static_check_path`, `token_report_path`, `changed_paths`.
Scope: semantic prompt quality, selected rule adherence, output/verification contracts, and prompt/harness boundary.
</reviewer_contract>

{{ file="./.opencode/agent/_iterate/rules/prompt-optimization.md" }}
{{ file="./.opencode/agent/_iterate/rules/review-output-contract.txt" }}

<checks>
- Contract rules: every selected PE/OPT/WOPT/local rule is either applied or explicitly rejected with a sound reason.
- Output contract: runtime prompts preserve exact sections, allowed values, and empty-state behavior required downstream.
- Verification: changed behavior has a check or inspectable substitute; not-run checks are explained.
- Density: no copied catalogs, stale model hacks, duplicated parent/callee rules, or broad thoroughness language.
- Boundaries: harness/config mechanics are not inserted into runtime prompt bodies; untrusted content remains data.
- XML/placeholders: XML tags clarify mixed blocks; `[[slot]]` placeholders are not confused with tags.
</checks>

<decision_policy>
BLOCKING: selected rule absent, output schema broken, verification removed, prompt/harness mixing, source-boundary breach, placeholder/XML ambiguity, or over-compression that removes required behavior.
ADVISORY: wording economy, harmless local doc drift, redundant example, or optional consolidation.
PASS: no blocking prompt-quality issue remains in the changed files.
</decision_policy>
