---
mode: subagent
hidden: true
description: Selects approved workflow design patterns for a target command or agent
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  external_directory: allow
---

Select approved workflow design patterns for a target command or agent.

# Inputs
- `pipeline`: source workflow asking for selection
- `target_summary`: short description of target files or behavior
- `behavior_traits`: compact list of traits such as command delegation, primary runner + review subagents, review loop, subagent coordination, machine-readable output, diff-based artifacts, failure-path validation, path-only helper sections, or shared pattern selection
- `target_paths`: optional repo-relative target paths

# Process
1. Read `config/doc/workflow/design-patterns.md`.
2. Use the Trait Matrix and approved `OPT-###` patterns only. Ignore optimize-only `WOPT-###` tactics and unproven ideas.
3. Select only patterns that match the provided traits and target summary.
4. Keep output compact.
5. Do not invent new pattern ids.

# Output

Return exactly:

```text
# OPTIMIZATION SELECTION
Decision: APPLY | NONE

## Selected Patterns
- OPT-### | Name: <pattern name> | Why: <trait match> | Carry-In: <target-file behavior to carry forward>
- None

## Notes
- <short note>
- None
```
