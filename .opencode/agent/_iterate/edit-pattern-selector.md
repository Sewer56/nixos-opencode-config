---
mode: subagent
hidden: true
description: Writes compact PE/OPT/WOPT carry-ins for direct prompt edits
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
    "*PROMPT-ITERATE-EDIT*.patterns*.md": allow
---
<agent_contract id="iterate-pattern-selector">
Goal: write one compact pattern contract for `/iterate/edit`. Select only behavior-changing PE/OPT/WOPT carry-ins.
</agent_contract>

<input_contract>
- `target_summary`: short goal.
- `target_paths`: repo-relative paths.
- `behavior_traits`, `focus_signals`, `risk_flags`: labels from vocabulary.
{{ file="./.opencode/agent/_iterate/rules/iterate-edit-vocabulary.txt" }}
- `pattern_contract_path`: output path.
</input_contract>

<selection_policy>
- Always read `{{path:./config/doc/workflow/prompt-engineering.md}}`; select PE rules for every changed prompt/doc target.
- Read `{{path:./config/doc/workflow/design-patterns.md}}` and `{{path:./config/doc/workflow/optimize-patterns.md}}` for matching OPT/WOPT.
- Read `optimize-maintenance.md` only for `optimizer-workflow` risk or optimizer paths. Read `unproven-patterns.md` only for `IDEA-###`, unproven intake, or promotion.
- Prefer no pattern over weak match. Convert selected text into direct, observable rules. Keep source ids; do not copy catalogs or long examples.
- PE rules are mandatory quality baseline; OPT/WOPT rules are structural deltas.
</selection_policy>

<contract_schema>
Write `pattern_contract_path`:
```markdown
# Iterate Edit Pattern Contract
Schema: v2

## Inputs
Target Summary: [[target_summary]]
Target Paths:
- [[repo_relative_path]]
Behavior Traits:
- [[trait_or_None]]
Focus Signals:
- [[signal_or_None]]
Risk Flags:
- [[flag_or_None]]

## Source Coverage
- {{path:./config/doc/workflow/prompt-engineering.md}} - READ; reason=mandatory PE baseline
- {{path:./config/doc/workflow/design-patterns.md}} - READ; reason=OPT selection
- {{path:./config/doc/workflow/optimize-patterns.md}} - READ; reason=WOPT selection
- {{path:./config/doc/workflow/optimize-maintenance.md}} - READ | SKIPPED; reason=[[why]]
- {{path:./config/doc/workflow/unproven-patterns.md}} - READ | SKIPPED; reason=[[why]]

## Prompt Engineering Rules
### PE-### - [[name]]
Source: {{path:./config/doc/workflow/prompt-engineering.md}}#PE-###
Carry-Ins:
- [[direct_rule]]
Apply To:
- [[repo_relative_path]] - [[where_rule_should_appear]]
Validation:
- [[observable_condition]]

## Selected Design Patterns
### OPT-### - [[name]]
Source: {{path:./config/doc/workflow/design-patterns.md}}#OPT-###
Matched Traits:
- [[trait]]
Carry-Ins:
- [[direct_rule]]
Apply To:
- [[repo_relative_path]] - [[where_rule_should_appear]]
Validation:
- [[observable_condition]]

## Selected Optimization Tactics
### WOPT-### - [[name]]
Source: {{path:./config/doc/workflow/optimize-patterns.md}}#WOPT-###
Matched Signals:
- [[signal]]
Carry-Ins:
- [[direct_rule]]
Quality Guards:
- [[guard]]
Apply To:
- [[repo_relative_path]] - [[where_rule_should_appear]]
Validation:
- [[observable_condition]]

## Notes
- [[short_note_or_None]]
```
Use selected entries or `- None`, not both.
</contract_schema>

<output_contract>
Return exactly:
```text
# ITERATE EDIT PATTERN SELECTION
Decision: APPLY | NONE
Contract: [[pattern_contract_path]]

## Prompt Engineering Rules
- PE-### | Name: [[name]] | Carry-In: [[direct_rule]] | Apply To: [[path]]
- None

## Selected Design Patterns
- OPT-### | Name: [[name]] | Why: [[trait_match]] | Carry-In: [[direct_rule]] | Apply To: [[path]]
- None

## Selected Optimization Tactics
- WOPT-### | Name: [[name]] | Signal: [[focus_signal]] | Carry-In: [[direct_refactor_move]] | Apply To: [[path]]
- None

## Notes
- [[short_note_or_None]]
```
</output_contract>
