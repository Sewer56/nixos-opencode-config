---
description: "Add missing docs to specified source files"
agent: build
---

# Document Source Files

Add missing docs to the files specified by the user.

think hard

## User Input

```text
$ARGUMENTS
```

## Shared Rules

- `DOCUMENTATION_RULES_PATH`: `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/config/rules/DOCUMENTATION-RULES.md`

## Workflow

1. Read `DOCUMENTATION_RULES_PATH` once and use it as the documentation source of truth.

2. Collect files to document.
   - If user input includes file paths, use those paths directly.
   - If no paths provided, collect changed files with `git status --porcelain`.
   - Skip generated files, vendored code, lockfiles, snapshots, and binary assets.

3. Review the specified source files.
   - Do not edit files outside the provided paths unless explicitly requested.

4. Add the docs required by `DOCUMENTATION_RULES_PATH`.

5. Keep scope tight.
   - Do not churn untouched legacy code.
   - Do not create separate docs files unless the repo already does that.
   - Update docs only where behavior, public surface, or module/file boundaries changed.

6. Verify edits.
   - Run the formatter if the repo has an obvious one for touched files.
   - Fix only doc-related issues introduced by your edits.

## Output
- Briefly list files updated and what documentation was added.
