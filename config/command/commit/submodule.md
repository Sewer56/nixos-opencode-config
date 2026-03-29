---
description: "Create semantic commit in submodule only"
agent: build
---

# Commit Submodule Changes

Create meaningful commits following `Keep a Changelog` format, but only inside a git submodule.
Additional user's instructions (if present) are below:

```
$ARGUMENTS
```

## Scope Rules (Strict)

1. Commit only inside the target submodule repository.
2. Do not create any commit in the parent repository.
3. Do not stage or commit the submodule pointer update in the parent repository.
4. Do not push unless the user explicitly asks.

## Process:

1. Identify the target submodule:
   - Prefer an explicit path provided in `$ARGUMENTS`.
   - Otherwise, detect changed submodule(s) from git status.
2. If no submodule changes are present, stop and report that there is nothing to commit.
3. Enter the submodule and inspect its diff.
4. Create commit messages based on Keep a Changelog categories:
   - **Added** for new features
   - **Changed** for changes in existing functionality
   - **Deprecated** for soon-to-be removed features
   - **Removed** for now removed features
   - **Fixed** for any bug fixes
   - **Security** for vulnerability fixes
5. Commit inside the submodule only.
6. Return the submodule path and resulting commit hash.

## Commit Execution

Write clear, human-readable commits. Only the first line is required; add context only when it helps.

**CRITICAL: Use heredoc for multiline commit messages to avoid quoting issues:**

```bash
git commit -F - <<'EOF'
Changed: Brief summary of the change

Optional: why the change matters in plain language.
EOF
```

## Guidelines

- Changelogs are for humans, not machines.
- Use clear, descriptive language; avoid jargon.
- Focus on what changed and why.
- Group related changes logically.
- Keep it brief: first line only unless context helps.

## Format (Keep a Changelog style)

```
Changed/Added/Fixed/Removed: <one-line summary>

Optional: brief description or bullet points if helpful.
```

Example (simple):
```
Fixed: Prevent accidental parent-repo commits
```

Example (with bullets when helpful):
```
Changed: Refactor authentication flow

- Move login logic into AuthService
- Add token refresh on 401 responses
```
