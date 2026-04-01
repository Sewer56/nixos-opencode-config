---
mode: primary
description: Builds orchestrator prompt packs from task descriptions
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*": deny
    "*PROMPT-ORCHESTRATOR.md": allow
    "*PROMPT-PRD-REQUIREMENTS.md": allow
    "*PROMPT-FINDING-*.md": allow
    "*PROMPT-??-*.md": allow
    "*PROMPT-??-*-PLAN.md": deny
    "*PROMPT-??-*-CODER-NOTES.md": deny
    "*PROMPT-??-*-REVIEW-LEDGER.md": deny
    "*PROMPT-ORCHESTRATOR.state.md": deny
    "*PROMPT-ORCHESTRATOR.validation.md": deny
    "*PROMPT-REQUIREMENTS-UNMET.md": deny
    "*PROMPT-SPLIT.md": deny
    "*PROMPT-DRAFT-*.md": deny
  question: allow
  grep: allow
  glob: allow
  list: allow
  todowrite: allow
  external_directory: allow
  task:
    "*": deny
    "codebase-explorer": allow
    "mcp-search": allow
    "_orchestrator/prompt-pack-reviewer": allow
    "_orchestrator/runner/requirements/requirements-preflight": allow
  # bash: deny
  # webfetch: deny
  # websearch: deny
  # codesearch: deny
  # lsp: deny
  # doom_loop: deny
  # skill: deny
---
