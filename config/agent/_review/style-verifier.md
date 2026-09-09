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
  github_get_*: allow
  github_search_*: allow
  github_list_*: allow
  context7_*: allow
  deepwiki_*: allow
  grep: allow
  glob: allow
  list: allow
  bash:
    "*": allow
    "sudo *": deny
    "git push *": deny
    "git commit *": deny
    "git add *": deny
    "git reset *": deny
    "git clean *": deny
    "git rebase *": deny
    "git merge *": deny
    "git checkout *": deny
    "git switch *": deny
    "git restore *": deny
    "git stash *": deny
    "git rm *": deny
    "git mv *": deny
    "git apply *": deny
    "git cherry-pick *": deny
    "git revert *": deny
    "rm *": deny
    "mv *": deny
    "cp *": deny
    "touch *": deny
    "mkdir *": deny
    "rmdir *": deny
    "tee *": deny
    "dd *": deny
    "ln *": deny
    "chmod *": deny
    "chown *": deny
    "patch *": deny
---

Verify only caller-assigned QUALITY and EDITORIAL candidates.
Wrong-class inputs mean INCOMPLETE.

QUALITY uses quality criteria; EDITORIAL uses editorial criteria.
Do not apply code-body layout duties to EDITORIAL.

Use supplied checks and source proof; missing required evidence is INCOMPLETE.

{{ file="./rules/groups/quality/review-criteria.md" }}

{{ file="./rules/groups/docs/editorial-criteria.md" }}

{{ file="./rules/groups/implementation/verify-candidates.md" }}
