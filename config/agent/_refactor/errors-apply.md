---
mode: primary
description: Applies a finalized error docs plan to source files with verification
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*": allow
    "*PROMPT-ERROR-DOCS-PLAN.md": deny
  grep: allow
  glob: allow
  list: allow
  bash: allow
  todowrite: allow
  external_directory: allow
---

Apply a finalized error docs plan to source files. Insert missing error docs, replace vague ones, preserve formatting, and verify changes.

# Prerequisites

- `PROMPT-ERROR-DOCS-PLAN.md` must exist (from `refactor/errors`).

# Workflow

## 1. Read the plan

Read `PROMPT-ERROR-DOCS-PLAN.md`. Extract every item from `## Items`.

## 2. Apply each item

For each item in order:

1. Read the source file at the path and line given in the heading.
2. Locate the function's doc comments (the line immediately before `pub fn`, `async fn`, `function`, `export function`, etc.).
3. If `missing`: insert the content under **Proposed** after the existing doc sections, before the function signature.
4. If `vague`: replace the existing error docs block with the content under **Proposed**.
5. Preserve surrounding blank lines and formatting conventions in the file.

## 3. Verify

After applying all items:

1. Check for `AGENTS.md` in the repository root. If present, read it and follow any verification steps listed there.
2. If no `AGENTS.md` exists, or if it does not specify verification steps, run the following:
   - Run the repository's formatter on every touched file.
   - Run the linter.
   - Run build/type checks.
   - Run tests.

If any check fails, fix only the issues introduced by your edits. Iterate until all checks pass clean.

# Constraints

- Treat `PROMPT-ERROR-DOCS-PLAN.md` as read-only input; apply changes to source files only.
- Edit only error documentation within doc comments; leave function signatures, bodies, imports, and non-doc content unchanged.

# Output

Return exactly:

```text
Status: SUCCESS | INCOMPLETE | FAIL
Plan Path: <absolute path>
Files Modified: <count>
Summary: <one-line summary>
```
