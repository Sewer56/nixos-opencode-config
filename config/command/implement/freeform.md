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
- Follow the plan's steps in order.
- Run formatter, linter, build, and tests after each cohesive group of changes.
- Iterate until all checks pass clean.

## 3. Review
- Spawn `_implement/freeform-reviewer`, passing inline:
  - `## Request`: original user request (verbatim or summarized)
  - `## Plan Summary`: what was planned from conversation context
  - `## Changes Made`: files changed and what was done in each
  - `## Notes`: additional context or `None`
- Wait for the response.

## 4. Loop
- After each review response, parse `Decision:` and `## Findings` from the inline `# REVIEW` block.
- If the response is malformed or missing the block: retry.
- If any findings (BLOCKING or ADVISORY): fix all, re-run reviewer with updated inline context.
- Repeat until `Decision: PASS` or 5 iterations reached.
- At cap with any findings remaining: FAIL.
- Before `Status: SUCCESS`:
  - Run one final audit with `_implement/freeform-reviewer` and updated inline context.
  - Parse current findings from its inline `# REVIEW` block.
  - If BLOCKING: fix, rerun touched, then re-audit.

## 5. Report
- Return final status. No auto-commit.

# Output
Return exactly:

```text
Status: SUCCESS | INCOMPLETE | FAIL
Iterations: <n>
Summary: <one-line summary>
```
