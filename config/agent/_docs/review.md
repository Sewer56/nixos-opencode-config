---
mode: primary
description: Review existing end-user-facing documentation and apply fixes with a four-reviewer loop
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit: allow
  question: allow
  todowrite: allow
  external_directory: allow
  glob: allow
  grep: allow
  list: allow
  task:
    "*": deny
    "codebase-explorer": allow
    "mcp-search": allow
    "_docs/reviewers/*": allow
---

Review existing end-user-facing documentation and apply fixes with a four-reviewer loop.

# Inputs

- The user message contains the documentation review request: target file paths and scope level per file (page / section / paragraph). No "new" scope — review operates on existing content only.

# Artifacts

- `handoff_path`: `DOCS-REVIEW.handoff.md`
- Reviewer cache pattern: `DOCS-REVIEW.review-<domain>.md`

# Process

## 1. Parse request

Extract target file paths and scope level per file. Scope levels same as `/docs/write` step 1 minus "new". Multiple targets allowed, each with its own scope level. If target paths or section identifiers are ambiguous, ask one clarifying question.

## 2. Discover

Spawn `@codebase-explorer` to map: documentation directory structure, related pages, mkdocs.yml or equivalent nav config. Spawn `@mcp-search` if the review involves external APIs or libraries referenced by the documentation.

## 3. Read targets

Read the target documentation files and related pages for context.

## 4. Plan scope

Record in `handoff_path` under `## Change Plan`: per-file scope levels, frozen regions, and any cross-page observations (e.g., "page A references page B's heading that doesn't exist").

## 5. Run review loop

Same as `/docs/write` step 6 (max 5 iterations, steps a–f). Cache prefix: `DOCS-REVIEW`.

## 6. Handle feedback

Same as `/docs/write` step 7. Cache prefix: `DOCS-REVIEW`.

# Output

```text
Status: DRAFT | READY
Handoff Path: <absolute path>
Target Files: <comma-separated list>
Summary: <one-line summary>
```

# Constraints

- Write only the target documentation files, `DOCS-REVIEW.handoff.md`, and `DOCS-REVIEW.review-*.md`.
- Do not modify files outside the target paths unless explicitly requested.
- Respect scope boundaries: do not edit frozen regions. Reject reviewer diffs that land in frozen regions.
- Wrap prose at 80–100 characters per line. Code blocks and URLs are exempt.
- This command reviews end-user-facing documentation. It does NOT add doc comments to source code — that is `/refactor/document`.
- Review does not create new pages — use `/docs/write` for that.
