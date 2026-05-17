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

- Derive `slug` from the request context as a 2–3 word identifier. Derive `artifact_base` as `PROMPT-DOCS-REVIEW-<slug>`.
- `handoff_path`: `artifact/<artifact_base>.handoff.md`
- Cache paths (written by reviewers, stored under `artifact/`):
  - `artifact/<artifact_base>.review-clarity.md`
  - `artifact/<artifact_base>.review-wording.md`
  - `artifact/<artifact_base>.review-engagement.md`
  - `artifact/<artifact_base>.review-consistency.md`
- Create parent directories unconditionally before writing any `artifact/...` path (mkdir -p semantics: no overwrite, no existence check).

# Focus

## Existing-doc scope
Review existing end-user-facing documentation only. Do not create new pages or source-code doc comments. Write only the target documentation files, `artifact/<artifact_base>.handoff.md`, and `artifact/<artifact_base>.review-*.md`. Do not modify files outside the target paths unless explicitly requested.

Bad: turn a section review into a new-page creation task.
Good: edit only requested existing docs within declared scope.

## Handoff and Delta
Write `handoff_path` before reviewers run. Delta entries must identify file, sections changed, scope level, and `Status: New | Changed | Unchanged`.

Bad: reviewers receive only target paths with no scoped Delta.
Good: reviewers receive `handoff_path` with per-file scope and status.

## Reviewer loop
Run clarity, wording, engagement, and consistency reviewers in parallel. Pass `handoff_path` and per-reviewer `cache_path`; reviewer prompts own their Focus, Process, and Output.

Bad: caller restates each reviewer's full rules.
Good: caller passes handoff path, cache_path, and lets reviewers read their own prompt.

## Frozen regions
Reject reviewer diffs that modify frozen regions.

Do not flag: reviewer findings outside frozen regions that preserve scope.

# Process

## 1. Parse request

Extract target file paths and scope level per file. Scope levels same as `/docs/write` step 1 minus "new". Multiple targets allowed, each with its own scope level. If target paths or section identifiers are ambiguous, ask one clarifying question.

## 2. Discover

Spawn `codebase-explorer` to map: documentation directory structure, related pages, mkdocs.yml or equivalent nav config. Spawn `mcp-search` if the review involves external APIs or libraries referenced by the documentation.

## 3. Read targets

Read the target documentation files and related pages for context.

## 4. Plan scope

Record in `handoff_path` under `## Change Plan`: per-file scope levels, frozen regions, and any cross-page observations (e.g., "page A references page B's heading that doesn't exist").

## 5. Run review loop

Max 5 iterations.

a. Write `handoff_path` with scope, per-file Delta, and Change Plan before first reviewer pass. Per-file Delta entries track: file path, sections changed, scope level.
   Include `Status: New | Changed | Unchanged` in each entry. Mark unchanged files as `Unchanged`.

b. Run four reviewers in parallel: `_docs/reviewers/clarity`, `_docs/reviewers/wording-cached`, `_docs/reviewers/engagement`, `_docs/reviewers/consistency-cached`. Pass only: `handoff_path` and `cache_path` (artifact/<artifact_base>.review-<domain>.md). Reviewers read the actual documentation files and use the handoff to determine which files and sections are in scope.

c. Validate each reviewer response: starts with `# REVIEW`, contains `Decision: PASS | ADVISORY | BLOCKING`, contains `## Findings` and `## Verified`. All 4 reviewers are diff-mandated — confirm each finding contains a unified diff block. Treat missing diffs as protocol violation requiring retry.

d. Record decisions in `handoff_path` for cross-domain arbitration. Apply domain ownership: CLARITY → clarity; WORDING → wording; ENGAGEMENT → engagement; CONSISTENCY → consistency.

e. Apply reviewer diffs via targeted edits; fall back to `Fix:` prose. Reject diffs in frozen regions (see Focus → Frozen regions).

f. Recompute Delta. Re-run all reviewers after every material revision (any substantive change to doc content — not cosmetic fixes like whitespace or typo corrections). Loop until no findings or 5 iterations.

   After a fix, rerun only reviewers whose domain changed. Do not rerun unrelated domains.

## 6. Handle feedback

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
