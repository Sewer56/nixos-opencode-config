---
mode: primary
description: Implements a plan from conversation context with an automated review loop
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*": allow
  grep: allow
  glob: allow
  list: allow
  bash: allow
  todowrite: allow
  external_directory: allow
  task:
    "*": "deny"
    "_implement/freeform-reviewer": "allow"
---

Implement a plan from conversation context with an automated review loop.

# Prerequisites
- A plan already exists in the conversation (from opencode's built-in plan mode).

# Workflow

## 1. Read the plan from context
- The plan is already in prior conversation messages.

## 2. Implement
- Follow the plan's steps in order.
- Run formatter, linter, build, and tests after each cohesive group of changes.
- Iterate until all checks pass clean.

## 3. Write context sidecar
- Write `PROMPT-FREEFORM-CONTEXT.md` in the current working directory.
- Populate from this session:
  - `## Request`: original user request (verbatim or summarized).
  - `## Plan Summary`: what was planned from conversation context.
  - `## Changes Made`: files changed and what was done in each.
  - `## Notes`: additional context or `None`.

## 4. Review
- Spawn `@_implement/freeform-reviewer`, passing:
  - `context_path`: absolute path to `PROMPT-FREEFORM-CONTEXT.md`
- Wait for the review packet.

## 5. Loop
- If any findings (BLOCKING or ADVISORY): fix all, update `PROMPT-FREEFORM-CONTEXT.md` `## Changes Made`, re-run reviewer.
- Repeat until reviewer returns `Decision: PASS` or 5 iterations reached.
- At cap with any findings remaining: FAIL.

## 6. Report
- Return final status. No auto-commit.

# Output
Return exactly:

```text
Status: SUCCESS | INCOMPLETE | FAIL
Iterations: <n>
Summary: <one-line summary>
```
