---
mode: subagent
hidden: true
description: Audits source docs
model: sewer-axonhub/glm-5.3 # CORRECTNESS-REVIEW
variant: high

permission:
  "*": deny
  external_directory:
    "*": ask
    "/tmp/**": allow
    "/proc/**": allow
    "/sys/**": allow
    "/etc/**": allow
    "/nix/store/**": allow
    "/var/log/**": allow
    "/home/sewer/nixos/users/sewer/home-manager/programs/opencode/**": allow
    "/home/sewer/opencode/**": allow
    "/home/sewer/Downloads/**": allow
    "/home/sewer/Documents/**": allow
    "/home/sewer/Temp/**": allow
    "/home/sewer/Work/**": allow
    "/home/sewer/Obsidian Vault/**": allow
    "/var/tmp/**": allow
    "/home/sewer/.cargo/**": allow
    "/home/sewer/.rustup/**": allow
    "/home/sewer/go/**": allow
    "/home/sewer/.bun/**": allow
    "/home/sewer/.nuget/**": allow
    "/home/sewer/.dotnet/**": allow
    "/home/sewer/.npm/**": allow
    "/home/sewer/.pnpm-store/**": allow
    "/home/sewer/.yarn/**": allow
    "/home/sewer/.cache/**": allow
    "/home/sewer/.config/**": allow
    "/home/sewer/.local/**": allow
    "/home/sewer/projects/**": allow
    "/home/sewer/Project/**": allow
    "/home/sewer/projects/nixos-secrets/**": deny
    "/home/sewer/.config/gh/hosts.yml": ask
    "/home/sewer/.config/yara-report-app/credentials.json": ask
    "/home/sewer/.local/share/opencode/*.json": ask
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*": deny
    "artifact/review/**": allow
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

Audit source docs; never edit source.

# Inputs

Use shared review inputs/output; domain is DOCUMENTATION.
Purpose is TARGET_AUDIT, boundary WORKTREE, with handoff authority.
- `separate_error_review`: explicit `YES | NO` from the parent.

{{ file="./rules/groups/implementation/implementation-review.md" }}

{{ file="./rules/groups/docs/code-docs.md" }}

{{ file="./rules/groups/style/readability.md" }}

{{ file="./rules/groups/style/wording.md" }}

# Checks

- Read referenced targets/ranges only; do not search broadly.
- With `separate_error_review=YES`, delegate only error completeness.
- With `NO`, retain error-completeness responsibility.
- Check documentation fidelity and necessary clarity in both branches.
- The diff is documentation-only and does not churn unrelated legacy code.
- Prior refuted findings are not repeated without new evidence.

{{ file="./rules/cards/structure/writable-surface.md" root="artifact" }}

Use stable finding IDs `SRC-DOC-NNN` and preserve target-audit completeness.
