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

**CRITICAL: Use heredoc for multiline commit messages to avoid quoting issues:**

```bash
git commit -F - <<'EOF'
Changed: Brief summary of the change

Description of what changed and why.

Changes:
- Specific change one
- Specific change two

Benefits:
- Benefit one
- Benefit two
EOF
```

## Guidelines:

- Changelogs are for humans, not machines.
- Use clear, descriptive commit messages.
- Focus on what changed and why.
- Group related changes logically.
- Be concise but informative.

## Format (Keep a Changelog style):
```
Changed/Added/Deprecated/Removed/Fixed/Security: <1 line change>

<Short description>

Changes:
- <Short bullet point>
- <Short bullet point>

Benefits:
- <Short bullet point>
- <Short bullet point>
```

Example:
```
Fixed: Correct submodule-only commit workflow for nested repo changes

Restrict commit execution to the target submodule to avoid accidental parent-repo commits.

Changes:
- Added strict parent-repo commit prohibition rules
- Added explicit submodule path and commit hash reporting

Benefits:
- Prevents accidental top-level history changes
- Makes submodule commit results easier to verify
```

Follow [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) principles for clear, maintainable commit history.
