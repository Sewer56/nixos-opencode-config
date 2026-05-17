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

- Derive `slug` from the request context as a 2–3 word identifier. Derive `artifact_base` as `PROMPT-DOCS-WRITE-<slug>`.
- `handoff_path`: `artifact/<artifact_base>.handoff.md`
- Cache paths (written by reviewers, stored under `artifact/`):
  - `artifact/<artifact_base>.review-clarity.md`
  - `artifact/<artifact_base>.review-wording.md`
  - `artifact/<artifact_base>.review-engagement.md`
  - `artifact/<artifact_base>.review-consistency.md`

# Focus

## Write scope
Write or modify end-user-facing documentation (guides, READMEs, mkdocs pages) only. Do not add doc comments to source code — that is `/refactor/document`. Write only the target documentation files, `artifact/<artifact_base>.handoff.md`, and `artifact/<artifact_base>.review-*.md`. Do not modify files outside the target paths unless explicitly requested.

## Frozen regions
Respect scope boundaries: do not edit frozen regions. Reject reviewer diffs that land in frozen regions.

# Process

## 1. Parse request

Extract target file paths, scope level per file, and intent (create or modify). Scope levels:

- **page**: the entire file may be rewritten. Agent has full freedom within the file.
- **section** (identified by heading): only the content under the specified heading may change. Everything above and below that heading is frozen.
- **paragraph** (identified by heading + position): only the specified paragraph may change. The rest of the section is frozen.
- **new**: create a new file from scratch. Full freedom.

Multiple targets allowed, each with its own scope level. If target paths or section identifiers are ambiguous, ask one clarifying question.

## 2. Discover

Spawn `codebase-explorer` to map: documentation directory structure, existing pages, mkdocs.yml or equivalent nav config, project README. Spawn `mcp-search` if the request involves external APIs or libraries the documentation must cover.

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
   Include `Status: New | Changed | Unchanged` in each entry. Mark unchanged files as `Unchanged`.

b. Run four reviewers in parallel: `_docs/reviewers/clarity`, `_docs/reviewers/wording-cached`, `_docs/reviewers/engagement`, `_docs/reviewers/consistency-cached`. Pass only: `handoff_path` and `cache_path` (artifact/<artifact_base>.review-<domain>.md). Reviewers read the actual documentation files and use the handoff to determine which files and sections are in scope.

c. Validate each reviewer response: starts with `# REVIEW`, contains `Decision: PASS | ADVISORY | BLOCKING`, contains `## Findings` and `## Verified`. All 4 reviewers are diff-mandated — confirm each finding contains a unified diff block. Treat missing diffs as protocol violation requiring retry.

d. Record decisions in `handoff_path` for cross-domain arbitration. Apply domain ownership: CLARITY → clarity; WORDING → wording; ENGAGEMENT → engagement; CONSISTENCY → consistency.

e. Apply reviewer diffs via targeted edits; fall back to `Fix:` prose. Reject diffs in frozen regions (see Focus → Frozen regions).

f. Recompute Delta. Re-run all reviewers after every material revision (any substantive change to doc content — not cosmetic fixes like whitespace or typo corrections). Loop until no findings or 5 iterations.

   After a fix, rerun only reviewers whose domain changed. Do not rerun unrelated domains.

## 7. Handle feedback

- On explicit confirmation:
  - Run final consistency audit with `_docs/reviewers/consistency-cacheless`.
  - Run final wording audit with `_docs/reviewers/wording-cacheless` only after late operational/protocol/command changes or prior wording BLOCKING findings.
  - Ignore caches and Delta shortcuts.
  - Return all current findings.
  - If BLOCKING: fix, recompute Delta, rerun touched reviewers, then re-audit.
  - If no BLOCKING: return `Status: READY`.
- On feedback: apply it, update Delta, and re-run the loop.
- Otherwise return `Status: DRAFT` with: "Re-review available — say 'review' to re-run reviewers."

# Output

Return exactly:

```text
Status: DRAFT | READY
Handoff Path: <absolute path>
Target Files: <comma-separated list>
Summary: <one-line summary>
```

# Constraints

- Wrap prose at 80–100 characters per line. Code blocks and URLs are exempt.
- Insert a blank line before the first bullet item when a list follows prose, and between adjacent bullet items whenever either wraps past one line; tight single-line lists (flags, enums) need no inter-item spacing.
