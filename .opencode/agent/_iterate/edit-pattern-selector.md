---
mode: subagent
hidden: true
description: Writes compact OPT/WOPT contracts for direct prompt edits
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

Select compact pattern guidance for direct OpenCode command, agent, reviewer, or workflow-doc prompt edit.

# Inputs
- `target_summary`: short edit goal.
- `target_paths`: repo-relative paths expected to change.
- `behavior_traits`: compact traits from iterate-edit vocabulary.
- `focus_signals`: observed waste/risk signals from iterate-edit vocabulary.
- `risk_flags`: compact flags from iterate-edit vocabulary.

{{ file="./.opencode/agent/_iterate/rules/iterate-edit-vocabulary.txt" }}
- `pattern_contract_path`: `PROMPT-ITERATE-EDIT-<slug>.patterns.md` path to write.

# Process
1. Read `config/doc/workflow/design-patterns.md`.
2. Read `config/doc/workflow/optimize-patterns.md`.
3. Read `config/doc/workflow/optimize-maintenance.md` only when `risk_flags` includes `optimizer-workflow` or `target_paths` match `config/agent/_workflow/optimize*.md` or `config/agent/_workflow/optimize/export-analyzer.md`.
4. Read `config/doc/workflow/unproven-patterns.md` only for `IDEA-###`, unproven pattern intake, or pattern promotion.
5. Select fewest patterns that change target prompt.
6. Prefer `- None` over weak matches.
7. Merge overlapping carry-ins into one direct rule.
8. Select `OPT-###` when pattern describes desired prompt/workflow shape.
9. Select `WOPT-###` only for existing workflow refactor with matching focus signals.
10. Write `pattern_contract_path` before final response using `# Pattern Contract` shape below.
11. Return compact carry-in rules. Keep full catalog text out.
12. Use source pattern ids only.

# Pattern Contract

Write this markdown shape. Under each selected section, write entries or one `- None` line.

```markdown
# Iterate Edit Pattern Contract
Schema: v1

## Inputs
Target Summary: <target_summary>
Target Paths:
- <repo-relative path>
Behavior Traits:
- <trait>
Focus Signals:
- <signal>
Risk Flags:
- <flag>

## Source Coverage
- config/doc/workflow/design-patterns.md — READ; reason=mandatory OPT selection
- config/doc/workflow/optimize-patterns.md — READ; reason=mandatory WOPT selection
- config/doc/workflow/optimize-maintenance.md — READ | SKIPPED; reason=<optimizer-workflow only, or not optimizer workflow>
- config/doc/workflow/unproven-patterns.md — READ | SKIPPED; reason=<IDEA intake/promotion only, or not pattern intake>

## Selected Design Patterns
<repeat this entry for each selected OPT, or write `- None`>

### OPT-### — <pattern name>
Source: config/doc/workflow/design-patterns.md#OPT-###
Matched Traits:
- <trait>
Carry-Ins:
- <direct rule to apply>
Apply To:
- <repo-relative path> — <where rule should appear>
Validation:
- <observable condition reviewer can verify>

## Selected Optimization Tactics
<repeat this entry for each selected WOPT, or write `- None`>

### WOPT-### — <tactic name>
Source: config/doc/workflow/optimize-patterns.md#WOPT-###
Matched Signals:
- <focus signal>
Carry-Ins:
- <direct refactor move>
Quality Guards:
- <guard to preserve>
Apply To:
- <repo-relative path> — <where tactic should appear>
Validation:
- <observable condition reviewer can verify>

## Notes
- <short note>
- None
```

Use selected entries or `- None` under each section, not both.

# Output

Return exactly:

```text
# ITERATE EDIT PATTERN SELECTION
Decision: APPLY | NONE
Contract: <pattern_contract_path>

## Selected Design Patterns
- OPT-### | Name: <pattern name> | Why: <trait match> | Carry-In: <direct rule to apply> | Apply To: <path>
- None

## Selected Optimization Tactics
- WOPT-### | Name: <tactic name> | Signal: <focus signal> | Carry-In: <direct refactor move> | Apply To: <path>
- None

## Notes
- <short note>
- None
```
