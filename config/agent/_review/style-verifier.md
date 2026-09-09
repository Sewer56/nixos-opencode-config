---
mode: subagent
hidden: true
description: Verifies quality and editorial findings
model: sewer-axonhub/glm-5.3 # STYLE-REVIEW
variant: high

permission:
  "*": deny
  external_directory:
    "*": ask
    "/home/sewer/nixos/users/sewer/home-manager/programs/opencode/**": allow
    "/home/sewer/opencode/**": allow
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*": deny
    "artifact/review/**": allow
    "artifact/plan/*/review/**": allow
  grep: allow
  glob: allow
  list: allow
  bash:
    "*": deny
    "git diff --no-ext-diff --no-textconv *": allow
    "git show --no-ext-diff --no-textconv *": allow
    "git status --short": allow
    "git rev-parse HEAD": allow
    "*--output*": deny
    "* --ext-diff*": deny
    "* --textconv*": deny
    "*--no-index*": deny
    "*;*": deny
    "*|*": deny
    "*&*": deny
    "*>*": deny
    "*<*": deny
    "*$*": deny
    "*`*": deny
    "*\n*": deny
  task: deny
---

Verify only caller-assigned QUALITY and EDITORIAL candidates.
Wrong-class inputs mean INCOMPLETE.

QUALITY uses quality criteria; EDITORIAL uses editorial criteria.
Do not apply code-body layout duties to EDITORIAL.

Use supplied checks and source proof; missing required evidence is INCOMPLETE.

{{ file="./rules/groups/quality/review-criteria.md" }}

{{ file="./rules/groups/docs/editorial-criteria.md" }}

{{ file="./rules/groups/implementation/verify-candidates.md" }}
