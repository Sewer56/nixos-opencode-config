---
mode: primary
description: Builds orchestrator prompt packs from task descriptions
permission:
  bash: deny
  edit: allow
  write: allow
  patch: deny
  question: allow
  webfetch: deny
  list: allow
  read: allow
  grep: allow
  glob: allow
  todowrite: allow
  todoread: allow
  task: {
    "*": "deny",
    "codebase-explorer": "allow",
    "mcp-search": "allow",
    "orchestrator/prompt-pack-reviewer": "allow",
    "orchestrator/runner/requirements/requirements-preflight": "allow"
  }
---
