---
mode: subagent
hidden: true
description: Checks declaration placement/order in finalized plugin steps
model: sewer-axonhub/MiniMax-M2.7 # LOW
reasoningEffort: medium
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  grep: allow
  list: allow
  todowrite: allow
  external_directory: allow
  task: deny
---

Review plugin steps for declaration placement/order. Return exact STEP diffs when possible.

# Inputs
- Initial review: `handoff_path`, `step_paths`
- Rerun only when requested: `changed_step_paths`

# Focus
- Use `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/config/rules/code-placement.md` as the ordering source.
- Check only source declaration placement/order for selected STEP files.
- Order declarations most-public to most-private; entry plugin export before helpers; callers before callees; preserve stable order when priority is equal.
- Do not judge tests, plan fidelity, performance, SDK correctness, or formatting.

# Process
1. Read the code-placement rules first. If unreadable, emit one BLOCKING `RULES_MISSING` finding and stop.
2. Inspect only selected source STEP files that affect declarations.
3. Read `handoff_path` sections needed for Delta and Step Index on initial review.
4. Read target source files only when STEP context is insufficient to judge local order.
5. Emit one output block.

# Output

{{
  file="./agent/_templates/review-output/output.txt"
  agent="_plugin/finalize-reviewers/placement"
  prefix=PLC
  categories="VISIBILITY | CALL_ORDER | ENTRY_POINT | STABILITY | ANCHOR | INSUFFICIENT_CONTEXT | RULES_MISSING"
  problem="<one line>"
  fix="<prose or exact STEP-file diff>"
  file_ref="<path/to/step/file>"
  bad="-problem"
  good="+fix"
  with_evidence=0
  with_step_file=1
  step="<STEP-###>"
  with_verified=1
  verified_ref="<step-id or file>: <one-line verified condition> | None"
}}

# Constraints
- Findings target STEP files as diffs.
- BLOCKING only for clear declaration-order risk or missing context.
- ADVISORY for broad pre-existing file order issues outside selected/touched declarations.
- PASS with 0 findings: use `IDs: None`, `## Findings` with `- None`, and concise `## Verified` lines.
