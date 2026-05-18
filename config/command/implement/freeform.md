---
description: "Implement a plan from conversation context with automated review loop"
agent: _implement/freeform
---

Implement a plan from conversation context with an automated review loop.

# Inputs
- Plan exists in prior conversation messages.

# Extra Instructions
$ARGUMENTS

# Workflow

## 1. Read plan
- Extract plan steps from conversation context.

## 2. Implement
- Follow plan steps in order.
- Run formatter, linter, build, and tests after each cohesive change group.
- Iterate until all checks pass clean.

## 3. Review
- Spawn `_implement/freeform-reviewer` with only:
  - `## Request`: original user request, verbatim or summarized.
  - `## Plan Summary`: planned work from conversation context.
  - `## Changes Made`: files changed and what changed in each.
  - `## Notes`: 0-2 current-run facts or `None`.
- Wait for the response.

## 4. Loop
- Parse `Decision:` and `## Findings` from the inline `# REVIEW` block.
- If the response is malformed or missing the block, retry.
- If any findings remain, fix them and re-run reviewer with updated run data.
- Repeat until `Decision: PASS` or 5 iterations.
- At cap with findings remaining, return `FAIL`.
- Before `Status: SUCCESS`, run one final audit with `_implement/freeform-reviewer` and updated run data.
- If final audit has BLOCKING findings, fix, rerun touched work, and re-audit.

## 5. Report
- Return final status. No auto-commit.

# Output
Return exactly:

```text
Status: SUCCESS | INCOMPLETE | FAIL
Iterations: <n>
Summary: <one-line summary>
```
