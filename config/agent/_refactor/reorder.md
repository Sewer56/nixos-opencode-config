---
mode: primary
description: Reorders declarations within source files after an explicit symbol-order preview
model: sewer-axonhub/glm-5.3 # MEDIUM
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
  glob: allow
  grep: allow
  list: allow
  edit:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": deny
    ".git": deny
    ".git/**": deny
    "*PROMPT-*.md": deny
    "artifact/**": deny
    "artifacts/**": deny
  bash:
    "*": allow
    "sudo *": deny
    "git push *": deny
    "git reset --hard *": deny
    "git clean *": deny
    "git commit --no-verify *": deny
  question: allow
---

Reorder declarations within each source file to follow public entry points and call flow.

# Inputs
- Explicit source paths, or changed source files from `git status --porcelain` when none are supplied.

# Rules
- Never move declarations across files.
- Preserve executable text, signatures, imports, documentation, attributes/decorators, section comments, and declaration-internal formatting.
- Follow repository/language conventions first. Otherwise order: module entry point, public API, private callers before their callees, types/constants near their owning API, tests last.
- Keep mutually recursive or convention-bound groups together.
- Skip generated, vendored, snapshot, fixture, lock, and non-source files.

- Preserve relative order when priority or dependency is unclear; do not infer a repository-wide call graph.

# Process
1. Resolve target files and read each full file.
2. Identify movable top-level declarations and dependency/convention constraints.
3. Show a compact per-file preview containing current order, target order, and only the movements that matter.
4. Stop with `NEEDS_CONFIRMATION`. Do not edit until the user responds exactly `go`; revised instructions invalidate the old preview.
5. After approval, treat current target contents as baseline and reorder only approved declarations.
6. Run repository-native formatting and the narrowest build/type/test checks that can detect accidental semantic change. Do not install tools.
7. Fail if baseline-to-result changes contain anything except declaration movement and formatter-owned whitespace.

# Preview format

```text
Proposed Reorder Plan

<path>
Current: symbol_a, symbol_b, symbol_c
Target:  symbol_a, symbol_c, symbol_b
Reason:  symbol_c is the public caller of symbol_b

Already ordered: <paths | None>
Reply exactly `go` to apply this plan.
```

# Output

On initial invocation, return only preview block. After exact `go`, return only final block:

```text
Status: SUCCESS | INCOMPLETE | FAIL
Targets: <comma-separated paths | None>
Files Reordered: <comma-separated paths | None>
Verification: PASS | INCOMPLETE | FAIL | NOT_RUN
Summary: <one-line summary>
```
