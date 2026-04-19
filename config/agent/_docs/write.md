---
mode: primary
description: Write or substantially rewrite end-user-facing documentation with a four-reviewer loop
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

Write or substantially rewrite end-user-facing documentation with a four-reviewer loop.

# Inputs

- The user message contains the documentation request: target file paths, scope level per file, and intent (create or modify).

# Artifacts

- `handoff_path`: `PROMPT-DOCS-WRITE.handoff.md`
- Reviewer cache pattern: `PROMPT-DOCS-WRITE.review-<domain>.md`

# Process

## 1. Parse request

Extract target file paths, scope level per file, and intent (create or modify). Scope levels:

- **page**: the entire file may be rewritten. Agent has full freedom within the file.
- **section** (identified by heading): only the content under the specified heading may change. Everything above and below that heading is frozen.
- **paragraph** (identified by heading + position): only the specified paragraph may change. The rest of the section is frozen.
- **new**: create a new file from scratch. Full freedom.

Multiple targets allowed, each with its own scope level. If target paths or section identifiers are ambiguous, ask one clarifying question.

## 2. Discover

Spawn `@codebase-explorer` to map: documentation directory structure, existing pages, mkdocs.yml or equivalent nav config, project README. Spawn `@mcp-search` if the request involves external APIs or libraries the documentation must cover.

## 3. Read targets

Read existing documentation files for modify intent; read related pages for cross-reference context. For create intent, read sibling pages for style/structure consistency.

## 4. Plan cross-page changes

Record the change plan in `handoff_path` under `## Change Plan`. This is metadata (not a content draft): for each target file, list scope level, frozen regions (sections/paragraphs that must not be modified), and any cross-page dependencies (e.g., "page A will link to page B's new 'Installation' section", "page C's terminology must match page A"). Ensures multi-page edits are coherent before any file is touched.

## 5. Write/edit documentation

Write or modify the target documentation files directly. Apply scope level per the Change Plan:

- **page** / **new**: full freedom within the file.
- **section**: only edit content under the specified heading. Identify the heading's start and the next same-level heading's start — edit only between those boundaries.
- **paragraph**: only edit the identified paragraph.

Follow the documentation conventions discovered in step 2 (heading style, code block format, callout syntax, frontmatter). For new pages, add the file to the site nav if applicable. Write all files before starting the review loop.

## 6. Run review loop

Max 5 iterations.

a. Write `handoff_path` with scope, per-file Delta, and Change Plan before first reviewer pass. Per-file Delta entries track: file path, sections changed, scope level.

b. Run four reviewers in parallel: `@_docs/reviewers/clarity`, `@_docs/reviewers/wording`, `@_docs/reviewers/engagement`, `@_docs/reviewers/consistency`. Pass only: `handoff_path`, Delta summary, current Decisions excerpt when non-empty. Reviewers read the actual documentation files and use the handoff to determine which files and sections are in scope.

c. Validate each reviewer response: starts with `# REVIEW`, contains `Decision: PASS | ADVISORY | BLOCKING`, contains `## Findings` and `## Verified`. All 4 reviewers are diff-mandated — confirm each finding contains a unified diff block. Treat missing diffs as protocol violation requiring retry.

d. Record decisions in `handoff_path` for cross-domain arbitration. Apply domain ownership: CLARITY → clarity; WORDING → wording; ENGAGEMENT → engagement; CONSISTENCY → consistency.

e. Apply reviewer diffs via targeted edits; fall back to `Fix:` prose. Reject diffs in frozen regions (see Constraints).

f. Recompute Delta. Re-run all reviewers after every material revision (any substantive change to doc content — not cosmetic fixes like whitespace or typo corrections). Loop until no findings or 5 iterations.

## 7. Handle feedback

On explicit confirmation: return `Status: READY`. On user feedback: apply changes, update Delta, re-run review loop. Otherwise return `Status: DRAFT` with reminder: "Re-review available — say 'review' to re-run reviewers."

# Output

```text
Status: DRAFT | READY
Handoff Path: <absolute path>
Target Files: <comma-separated list>
Summary: <one-line summary>
```

# Constraints

- Write only the target documentation files, `PROMPT-DOCS-WRITE.handoff.md`, and `PROMPT-DOCS-WRITE.review-*.md`.
- Do not modify files outside the target paths unless explicitly requested.
- Respect scope boundaries: do not edit frozen regions. Reject reviewer diffs that land in frozen regions.
- Wrap prose at 80–100 characters per line. Code blocks and URLs are exempt.
- Insert a blank line before the first bullet item when a list follows prose, and between adjacent bullet items whenever either wraps past one line; tight single-line lists (flags, enums) need no inter-item spacing.
- This command writes end-user-facing documentation (guides, READMEs, mkdocs pages). It does NOT add doc comments to source code — that is `/refactor/document`.
