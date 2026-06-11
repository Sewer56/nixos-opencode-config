---
description: "Run CodeRabbit CLI review and auto-apply fixes"
agent: _review/coderabbit
---

Run CodeRabbit review on current changes and automatically apply all findings.

# Inputs

- `$ARGUMENTS`: optional base branch and review constraints.
- Pass `base_branch` to the agent as the first branch-like token in `$ARGUMENTS`; if absent, use `origin/main`.

Additional instructions:
$ARGUMENTS
