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

- `handoff_path`: `PROMPT-DOCS-REVIEW.handoff.md`
- Reviewer cache pattern: `PROMPT-DOCS-REVIEW.review-<domain>.md`

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

Max 5 iterations.

a. Write `handoff_path` with scope, per-file Delta, and Change Plan before first reviewer pass. Per-file Delta entries track: file path, sections changed, scope level.

b. Run four reviewers in parallel: `@_docs/reviewers/clarity`, `@_docs/reviewers/wording`, `@_docs/reviewers/engagement`, `@_docs/reviewers/consistency`. Pass only: `handoff_path`, Delta summary, current Decisions excerpt when non-empty. Reviewers read the actual documentation files and use the handoff to determine which files and sections are in scope.

c. Validate each reviewer response: starts with `# REVIEW`, contains `Decision: PASS | ADVISORY | BLOCKING`, contains `## Findings` and `## Verified`. All 4 reviewers are diff-mandated — confirm each finding contains a unified diff block. Treat missing diffs as protocol violation requiring retry.

d. Record decisions in `handoff_path` for cross-domain arbitration. Apply domain ownership: CLARITY → clarity; WORDING → wording; ENGAGEMENT → engagement; CONSISTENCY → consistency.

e. Apply reviewer diffs via targeted edits; fall back to `Fix:` prose. Reject diffs in frozen regions (see Constraints).

f. Recompute Delta. Re-run all reviewers after every material revision (any substantive change to doc content — not cosmetic fixes like whitespace or typo corrections). Loop until no findings or 5 iterations.

## 6. Handle feedback

On explicit confirmation: return `Status: READY`. On user feedback: apply changes, update Delta, re-run review loop. Otherwise return `Status: DRAFT` with reminder: "Re-review available — say 'review' to re-run reviewers."

# Output

```text
Status: DRAFT | READY
Handoff Path: <absolute path>
Target Files: <comma-separated list>
Summary: <one-line summary>
```

# Constraints

- Write only the target documentation files, `PROMPT-DOCS-REVIEW.handoff.md`, and `PROMPT-DOCS-REVIEW.review-*.md`.
- Do not modify files outside the target paths unless explicitly requested.
- Respect scope boundaries: do not edit frozen regions. Reject reviewer diffs that land in frozen regions.
- Wrap prose at 80–100 characters per line. Code blocks and URLs are exempt.
- This command reviews end-user-facing documentation. It does NOT add doc comments to source code — that is `/refactor/document`.
- Review does not create new pages — use `/docs/write` for that.
