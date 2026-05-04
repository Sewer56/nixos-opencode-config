---
mode: subagent
hidden: true
description: Checks code-adjacent error documentation coverage and specificity for finalized steps (cached)
model: sewer-axonhub/MiniMax-M2.7  # LOW
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
  todowrite: allow
  edit:
    "*PROMPT-PLAN*.review-codedoc-errors.md": allow
  external_directory: allow
---

{{ file="./agent/_plan/finalize-codedoc-reviewers/errors-shared-pre.txt" }}

# Process

1. Load `handoff_path` sections: `## Delta`, `## Review Ledger`, and non-empty `### Decisions`. Load cache by replacing `.handoff.md` with `.review-codedoc-errors.md`; missing/malformed cache is empty.
2. Inspect Changed/New I#/T# steps, own Open findings, and decision-referenced items; carry forward Verified entries only for Unchanged Delta items.
3. Read selected step files in one batch. Open referenced source files only when the step diff lacks context for public API status or reachable error variants.
4. Check Open→Resolved transitions. Update only changed cache entries, preserving unchanged cache text byte-for-byte, then emit the `# REVIEW` block. On malformed-output retry without new Delta/Decision entries, reuse prior analysis/cache and re-emit valid output.

In the `# REVIEW` output, set `Agent:` to `_plan/finalize-codedoc-reviewers/errors-cached`.

{{ file="./agent/_plan/finalize-codedoc-reviewers/errors-cached-post.txt" }}
