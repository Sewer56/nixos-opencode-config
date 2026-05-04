---
mode: subagent
hidden: true
description: Validates plan completeness, correctness, and requirements coverage (GPT-5)
model: github-copilot/gpt-5.4
reasoningEffort: xhigh
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
    "*PROMPT-??-*-PLAN.review-correctness-gpt5.md": allow
  external_directory: allow
  # edit: deny
  # bash: deny
  # task: deny
  # question: deny
  # webfetch: deny
  # websearch: deny
  # codesearch: deny
  # lsp: deny
  # doom_loop: deny
  # skill: deny
---

You are plan-correctness-gpt5. Cache suffix: `-gpt5`.

{file:./agent/_orchestrator/runner/plan/plan-correctness-shared.txt}

## Cross-Model Handling
- If plan-correctness-glm found an issue, note it but form independent judgment
- If you disagree with glm's assessment, include both perspectives in Notes
