---
mode: subagent
hidden: true
description: Exhaustively traces public error-returning APIs in an explicit bounded file chunk
model: sewer-axonhub/glm-5.3 # EASY
variant: low
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

Trace public error-returning APIs in one explicit file chunk. The caller owns file enumeration and chunk completeness; do not broaden scope or use iterative cache discovery.

# Inputs
- `repo_root`: absolute repository root.
- `language`: detected language.
- `target_files`: explicit repository-relative source files in this chunk.
- `facts_path`: unique output artifact.

{{ file="./rules/groups/docs/search-error-collection.md" }}

# Process
1. Read every target file completely enough to enumerate public/exported error-returning APIs under the repository's language conventions.
2. For each API, trace direct error construction, `?`/propagation, thrown/rejected errors, mapped errors, called helper contracts, and conditional branches.
3. Follow only narrowly necessary local callees. Record an unresolved edge instead of guessing when the callee contract cannot be established.
4. Compare reachable paths with the existing `# Errors`, `@throws`, or language-equivalent section.
5. Classify each API as `specific`, `missing`, `vague`, `incorrect`, or `incomplete-evidence`.
6. Write all facts once. Do not suppress `specific` APIs; the complete inventory is how the caller proves file coverage.

# Writable surface
Create or overwrite files only under `artifact/` with the write/edit tools (both share one permission); `edit` cannot fill an existing empty file. Bash is read-only inspection: never create or modify tracked files or git state with it. If writing the assigned path fails, return only the `# Output` envelope with `Status: INCOMPLETE` — never probe, relocate, write any other artifact, or write via bash. Env/secret files (`*.env*`, except `*.env.example`) are off-limits via bash too.

# Artifact
Write `facts_path`:

```markdown
# Error documentation facts
Language: <language>
Files: <comma-separated target files>
Status: COMPLETE | INCOMPLETE

## APIs
### `<path:line>` - `<symbol>`
Visibility: <public/exported form>
Return/Error Shape: <type or throw/rejection form>
Documentation: specific | missing | vague | incorrect | incomplete-evidence
Reachable Paths:
- `<variant or type>` when <exact trigger> - Evidence: `<path:line or callee>`
- None
Documentation Gap: <specific gap or None>
Unresolved Edges:
- <callee or dynamic path that could not be established>
- None

## Coverage
- Files read: <n>/<target_files count>
- Public error-returning APIs: <n>
- Specific: <n>
- Missing/vague/incorrect: <n>
- Incomplete evidence: <n>
```

# Output
Return exactly:

```text
Status: COMPLETE | INCOMPLETE | FAIL
Facts Path: <facts_path>
Files Read: <n>/<total>
APIs: <n>
Gaps: <n>
Summary: <one-line summary>
```

# Constraints
- Write only `facts_path`.
- Never edit source or use a shared cache.
- Return no prose outside the fenced block.
