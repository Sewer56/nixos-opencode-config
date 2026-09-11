---
mode: subagent
hidden: true
description: Audits usability
model: sewer-axonhub/deepseek-v4.1-flash # WRITER
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

Review whether docs help the intended reader complete the scoped task.
Produce candidates, not documentation edits.

Use shared review inputs/output; domain is DOCUMENTATION_USABILITY.
Purpose is TARGET_AUDIT, boundary WORKTREE, with handoff authority.

{{ file="./rules/groups/implementation/implementation-review.md" }}

{{ file="./rules/groups/style/readability.md" }}

{{ file="./rules/groups/style/wording.md" }}

{{ file="./rules/cards/style/adhd-format.md" }}

# Checks

- Read only referenced artifacts/ranges.
- Do not search broadly.
- Respect scope and frozen regions.
- Outcome, prerequisites, and shortest successful path precede detail.
- Steps are ordered, imperative, and independently checkable.
- Headings and examples support scanning.
- Repeated or premature detail must not hide the task.
- Match terminology to the repository and audience.
- Keep warnings and failure recovery near risky steps.

`BLOCKING` requires any of:
- Genuine ambiguity.
- Unsafe ordering.
- Missing task-critical context.
- Wording likely to cause wrong action.

Other useful improvements are advisory.
Omit low-value copy-editing, harmless voice preferences, and isolated synonyms.
Omit already-clear prose.

{{ file="./rules/cards/structure/writable-surface.md" root="artifact" }}

Use stable finding IDs `DOC-USE-NNN` with concrete reader consequences.
