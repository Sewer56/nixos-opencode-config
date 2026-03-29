---
description: "Create semantic commits following changelog format"
agent: build
---

# Commit Changes

Write a clear commit for the current changes. Additional user instructions (if present):

```
$ARGUMENTS
```

## Process

1. Run `git diff` to understand what changed
2. **Submodule check**: If changes are in a submodule directory:
   - Commit and push inside the submodule first
   - Return and stage the submodule pointer update
3. Write the commit message using a Keep a Changelog prefix:
    - **Added** - new features
    - **Changed** - changes to existing functionality
    - **Deprecated** - soon-to-be removed features
    - **Removed** - now-removed features
    - **Fixed** - bug fixes
    - **Security** - vulnerability fixes

## Commit format

Write clear, human-readable commits. Only the first line is required; add context only when it helps.

Use a heredoc for multiline messages:

```bash
git commit -F - <<'EOF'
Changed: Short summary of what changed

Optional: brief description or bullet points when helpful.
EOF
```

## Example (simple)

```
Fixed: Handle missing config file gracefully
```

## Example (with bullets when helpful)

```
Changed: Refactor authentication flow

- Move login logic into AuthService
- Add token refresh on 401 responses
```

## Guidelines

- One logical change per commit
- Say what changed and why, not how
- Reference issues/tickets when applicable
- Run tests before committing

Follow [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) for more on the prefix categories.
