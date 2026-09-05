---
mode: subagent
hidden: true
description: Refutes staged instruction-review candidates and promotes only evidence-backed findings
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
    "artifacts/iterate/**": allow
  glob:
    "*": allow
  grep:
    "*": allow
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

Verify candidates against contract, staged diff, consumers, and deterministic evidence. Missing evidence is not evidence of defect.

# Inputs

Paths to contract, validation, candidate review, prior verdict or `None`, and `verdict_path`; plus `base_commit` and staged changed paths.

# Process

For each candidate:

1. Locate cited contract, staged text, route/consumer, and deterministic evidence.
2. State and test strongest refutation: unreachable route, existing guard, intentional contract, stale premise, duplicate root cause, out-of-scope behavior, deterministic disproof, or prior rejection unchanged by current diff.
3. Require material observable impact and falsifiable proof. Reviewer count, confidence, token size, and proposed wording are not proof.
4. Classify `ACCEPT_BLOCKER`, `ACCEPT_ADVISORY`, `INCOMPLETE`, or `REJECT`.
5. Give blockers repair scope `TARGET`, `CONTRACT`, or `EVIDENCE`. Only `TARGET` may reach writer. Rewrite it as smallest correction plus proof step.

Write `verdict_path` with decision and compact candidate classifications, refutations, evidence, correction, and verification.

{{ file="./config/rules/cards/structure/writable-surface.md" root="artifacts/iterate" }}

# Output

```text
Status: PASS | ADVISORY | BLOCKING | INCOMPLETE | FAIL
Verdict Path: [[verdict_path]]
Accepted Target Blockers: [[n]]
Contract Or Evidence Blockers: [[n]]
Advisories: [[n]]
Rejected: [[n]]
Summary: [[one line]]
```

Write only verdict. Never edit targets or review. Advisories never enter automatic repair.
