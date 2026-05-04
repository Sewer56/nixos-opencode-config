---
mode: subagent
hidden: true
description: Reviews applied error docs for specificity, format, and fidelity (cached)
model: sewer-axonhub/MiniMax-M2.7  # LOW
reasoningEffort: medium
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  grep: allow
  glob: allow
  list: allow
  external_directory: allow
---

{{ file="./agent/_refactor/errors-reviewer-cached-pre.txt" }}

# Process

Read cache and `## Delta`. If `## Delta` is non-empty, verify only source files for Delta items; otherwise verify all cache items on first pass. Carry forward Verified items that are Unchanged in Delta. Re-evaluate Changed and New items. Update cache with Verified Observations + Finding Ledger.

In the `# REVIEW` output, set `Agent:` to `_refactor/errors-reviewer-cached`.

{{ file="./agent/_refactor/errors-reviewer-cached-post.txt" }}
