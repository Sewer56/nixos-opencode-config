---
description: "Create semantic commits following changelog format"
agent: build
---

# Commit Changes

Create meaningful commits following `Keep a Changelog` format.
Additional user's instructions (if present) are below:

```
$ARGUMENTS
```

## Process:

1. Check current git diff to understand changes
2. Create commit messages based on Keep a Changelog categories:
   - **Added** for new features
   - **Changed** for changes in existing functionality  
   - **Deprecated** for soon-to-be removed features
   - **Removed** for now removed features
   - **Fixed** for any bug fixes
   - **Security** for vulnerability fixes

## Guidelines:

- Changelogs are for humans, not machines.
- Use clear, descriptive commit messages
- Focus on what changed and why
- Group related changes logically
- Commit when ready
- Be concise but informative

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
Changed: Refine commit template for changelog-style messages

Use a structured body so commit history is easier to scan and understand.

Changes:
- Added explicit Changes and Benefits sections
- Standardized allowed Keep a Changelog categories

Benefits:
- Makes commit intent clearer at a glance
- Improves readability of project history
```

Follow [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) principles for clear, maintainable commit history.
