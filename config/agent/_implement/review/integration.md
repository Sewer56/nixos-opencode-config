---
mode: subagent
hidden: true
description: Reviews the complete implementation for cross-cohort correctness, acceptance coverage, and integration drift
model: sewer-axonhub/glm-5.3 # HARD
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
    "artifact/**": allow
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

Review the complete base-to-final implementation as one system. Focus on interactions that isolated cohort reviews can miss.

# Inputs
- `plan_path`, `handoff_path`.
- `base_commit`: implementation start commit.
- `scope`: committed `base_commit..HEAD` or staged repair against `base_commit`.
- `changed_paths`.
- `validation_path`: latest full validation ledger.
- `review_path`.
- `prior_verdict_paths`: prior final verdicts or `None`.

{{ file="./rules/groups/implementation/implementation-review.md" }}

{{ file="./rules/groups/implementation/review-findings.md" }}

# Review
Apply imported rules to cumulative base-to-final diff.

Audit acceptance and impact map, then inspect only cross-cohort composition: end-to-end contracts, unchanged consumers, registrations/exports/migrations, cleanup/rollback, compatibility, and integration evidence.

Do not repeat cohort findings unless cumulative evidence changes conclusion.

{{ file="./rules/cards/structure/writable-surface.md" root="artifact" }}

# Artifact
Write `review_path`:

```markdown
# Candidate Review
Domain: INTEGRATION
Scope: FINAL
Base Commit: <base_commit>
Decision: PASS | CANDIDATES | INCOMPLETE

## Findings
### [INT-NNN]
Proposed Severity: BLOCKING | ADVISORY
Requirement: <AC/INV/P id or cross-component contract>
Location: `<path:line>` / `<path:symbol>` / `Multiple`
Claim: <one falsifiable integration claim>
Evidence Type: EXECUTED | STATIC | CODE_PATH | CONTRACT
Evidence: <cross-file/tree/tool evidence>
Failure Path: <end-to-end state/input -> changed components -> incorrect result>
Impact: <observable integration, compatibility, or acceptance failure>
Verification: <specific integration check or falsifiable reasoning>
Smallest Fix:
<bounded cross-component correction; include a short fenced code block when exact integration shape matters>
- None

## Acceptance Audit
| Acceptance | Status     | Proof      | Evidence                          |
| ---------- | ---------- | ---------- | --------------------------------- |
| AC-1       | SATISFIED  | EXECUTED   | <test/command and result>         |
| AC-2       | SATISFIED  | STATIC     | <path/symbol/contract proof>      |
| AC-3       | INCOMPLETE | INCOMPLETE | <missing environment or evidence> |

## Verified
- <cross-cohort behavior checked and correct>
- None

## Notes
- <limitations>
- None
```

# Output
Return exactly:

```text
Status: PASS | CANDIDATES | INCOMPLETE | FAIL
Domain: INTEGRATION
Review Path: <review_path>
Finding Count: <n>
Summary: <one-line summary>
```
