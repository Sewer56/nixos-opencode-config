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
   - **Added** — new features
   - **Changed** — changes to existing functionality
   - **Deprecated** — soon-to-be removed features
   - **Removed** — now-removed features
   - **Fixed** — bug fixes
   - **Security** — vulnerability fixes

## Commit format

Use a heredoc so multiline messages stay clean:

```bash
git commit -F - <<'EOF'
Changed: Short summary of what changed and why

Longer description if the change needs context.
Keep this optional — skip it for small changes.

Changes:
- Bullet point (optional, useful for larger changes)

Benefits:
- Bullet point (optional, useful for larger changes)
EOF
```

The only required part is the first line (`Category: summary`). Add a body, Changes, or Benefits only when they add real value.

## Example

```
Fixed: Handle missing config file gracefully

The app crashed on startup when config.json was absent.
Now it creates a default config instead.

Changes:
- Added fallback to default config on missing file
- Removed hard exit on parse error
```

## Guidelines

- One logical change per commit
- Say what changed and why, not how
- Reference issues/tickets when applicable
- Run tests before committing

Follow [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) for more on the prefix categories.
