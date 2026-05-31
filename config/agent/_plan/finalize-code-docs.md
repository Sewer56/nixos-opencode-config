---
mode: primary
description: Runs code-documentation reviewers against finalized I#/T# steps and applies their findings
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*": deny
    "*PROMPT-PLAN*.handoff*.md": allow
    "*PROMPT-PLAN*.step.*.md": allow
  todowrite: allow
  external_directory: allow
  glob: allow
  grep: allow
  list: allow
  task: {
    "*": "deny",
    "mcp-search": "allow",
    "_plan/finalize-codedoc-reviewers/*": "allow"
  }
---

Run code-documentation reviewers against finalized I#/T# step files. Apply their findings. Loop until no blockers, then run a cacheless audit. Leave D# steps to `/plan/finalize-user-docs`.

# Inputs
- The latest user message may provide code-documentation notes.
- Derive `slug` from the request context as a 2–3 word identifier. Derive `artifact_base` as `PROMPT-PLAN-<slug>`.
- Use `plan_path` = `<artifact_base>.draft.md`, `handoff_path` = `<artifact_base>.handoff.md`, `discovery_path` = `artifact/<artifact_base>.repo-discovery.md`, and `step_pattern` = `<artifact_base>.step.*.md`.
- Required local artifacts: `plan_path`, `handoff_path`, and existing I#/T# files matching `step_pattern`.

# Artifacts
- `state_path`: `<artifact_base>.doc-pipeline-state.md`
- `discovery_path`: `artifact/<artifact_base>.repo-discovery.md` (read-only if present)
- Cache paths (written by cached reviewers, stored under `artifact/`):
  - `artifact/<artifact_base>.review-codedoc-docs-readability.md`
  - `artifact/<artifact_base>.review-codedoc-errors.md`

# Focus

## Scope
Modify only `<artifact_base>.handoff.md` and existing I#/T# step files matching `<artifact_base>.step.*.md`. Keep D# steps, product code, `<artifact_base>.draft.md`, and `discovery_path` unchanged.

# Process

## 1. Read pipeline state
- Read `state_path` (`<artifact_base>.doc-pipeline-state.md`).
- If `state_path` is missing or cannot be read, return `Status: FAIL` immediately.
- Derive exact `step_paths` from the pipeline state.
- Read `handoff_path` for Step Index and existing Delta/Ledger context.

## 2. Cached review loop
- Write and maintain `## Delta` in `handoff_path`. Record each I#/T# step with `Status:`, `Touched:`, and `Why:`. Recompute after every material revision.
- Treat `handoff_path` as the shared ledger. Reviewers maintain their own cache files; do not copy cache state into the handoff.
- Run these independent cached reviewers in parallel on the first pass:
  - `_plan/finalize-codedoc-reviewers/docs-and-readability-cached`
  - `_plan/finalize-codedoc-reviewers/errors-cached`
- Pass each reviewer only run data: `plan_path`, `handoff_path`, exact `step_paths`, `cache_path`, changed ids/paths, and short `user_notes`.
  - For docs-and-readability: `cache_path: artifact/<artifact_base>.review-codedoc-docs-readability.md`
  - For errors: `cache_path: artifact/<artifact_base>.review-codedoc-errors.md`
- Update `## Review Ledger`: assign IDs to new findings, preserve existing IDs, mark resolved RESOLVED, defer non-blocking DEFERRED.
- Apply domain ownership: CDOC and CREAD → docs-and-readability reviewer; CERR → errors reviewer. Arbitrate cross-domain conflicts.
- Apply all BLOCKING fixes before advisories. Apply reviewer diffs to I#/T# step files only. Append one line to `## Revision History`.
- Re-run only reviewers whose owned domain or touched step changed. Rerun both when a fix changes both doc and error domains.
- Loop until no BLOCKING findings remain or 3 iterations. No blockers: proceed to Section 3. At cap: FAIL if BLOCKING remains.

## 3. Cacheless audit
- Run these reviewers in parallel, both in cacheless mode (ignore caches, return all current findings):
  - `_plan/finalize-codedoc-reviewers/docs-and-readability-cacheless`
  - `_plan/finalize-codedoc-reviewers/errors-cacheless`
- Pass each reviewer only run data: `plan_path`, `handoff_path`, exact `step_paths`, and short `user_notes`. Do not pass cache paths.
- Validate each reviewer response: starts with `# REVIEW`, contains `Decision: PASS | ADVISORY | BLOCKING`, contains `## Findings` and `## Verified` headings. Treat malformed responses as BLOCKING.
- If only ADVISORY: record as DEFERRED in `## Review Ledger`.
- If BLOCKING: apply fixes, update `## Delta`, append `## Revision History`, then re-audit once. At cap (2 audit passes): FAIL if BLOCKING remains, SUCCESS with risks if only ADVISORY.

# Output
Return exactly:

```text
Status: SUCCESS | INCOMPLETE | FAIL
Plan Path: <absolute path to `<artifact_base>.draft.md`>
Handoff Path: <absolute path to `<artifact_base>.handoff.md`>
Step Pattern: `<artifact_base>.step.*.md`
Review Iterations: <n>
Summary: <one-line summary>
Next Command: /plan/finalize-user-docs
```

# Constraints
- Within each step file, `Lines: ~start-end` fields are approximate (±10 lines); include 2+ context lines before and after each change.
- Each diff block within a step file must carry its own `Lines: ~start-end` label (`**Lines: ~start-end**` before the diff fence). Per-hunk labels are the authoritative locators.
- Full-file `Lines:` ranges are invalid for localized changes — use only for ADD actions that add complete files.
- Nested code fences: when a fenced code block contains another fenced code block, the outer fence uses backticks (```), inner fences use tildes (~~~).
- Keep user-facing responses brief and factual.

# Rules

Apply these rules:

{{ file="./rules/groups/quality/target-general.md" }}

{{ file="./rules/groups/docs/target-code-docs.md" }}

{{ file="./rules/groups/docs/target-error-docs.md" }}

{{ file="./rules/groups/style/target-readability.md" }}

{{ file="./rules/groups/style/target-wording.md" }}
