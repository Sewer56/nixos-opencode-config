---
description: "Add missing docs to uncommitted source files"
agent: build
---

# Document Changed Code

Add missing docs to uncommitted source files only.

think hard

## User Input

```text
$ARGUMENTS
```

## Shared Rules

- `DOC_RULES_PATH`: `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/config/rules/ORCHESTRATOR-PLANNING-RULES.md`

## Workflow

1. Read `DOC_RULES_PATH` once and use it as the documentation source of truth.

2. Collect changed files with `git status --porcelain`.
   - Include staged, unstaged, and untracked files.
   - If user input includes paths, intersect with the changed-file set.
   - If no matching files exist, stop and say nothing needs documentation.

3. Review only matching source files.
   - Skip generated files, vendored code, lockfiles, snapshots, and binary assets.
   - Do not edit files without uncommitted changes.

4. Add the docs required by `DOC_RULES_PATH`.

5. Keep scope tight.
   - Do not churn untouched legacy code.
   - Do not create separate docs files unless the repo already does that.
   - Update docs only where behavior, public surface, or module/file boundaries changed.

6. Verify edits.
   - Run the formatter if the repo has an obvious one for touched files.
   - Fix only doc-related issues introduced by your edits.

## Output
- Briefly list files updated and what documentation was added.
