---
mode: primary
description: Reviews and revises code-adjacent documentation in finalized implementation/test steps
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

Review and revise code-adjacent documentation (API references, doc comments, inline comments inside non-trivial code bodies, parameter descriptions, error message strings, developer-facing READMEs) inside finalized code/test steps. Apply documentation and error-doc fixes to existing Implementation (I#) and Test (T#) step files. Leave end-user documentation steps (D# steps) to `/plan/finalize-user-docs`.

# Inputs
- The latest user message may provide code-documentation notes.
- Derive `slug` from the request context as a 2–3 word identifier. Derive `artifact_base` as `PROMPT-PLAN-<slug>`.
- Use `plan_path` = `<artifact_base>.draft.md`, `handoff_path` = `<artifact_base>.handoff.md`, `discovery_path` = `artifact/<artifact_base>.repo-discovery.md`, and `step_pattern` = `<artifact_base>.step.*.md`.
- Required local artifacts for this run: `plan_path`, `handoff_path`, and existing I#/T# files matching `step_pattern`.
- Read `discovery_path` when it exists; treat it as read-only shared repo context.

# Artifacts
- `state_path`: `<artifact_base>.doc-pipeline-state.md`
- `discovery_path`: `artifact/<artifact_base>.repo-discovery.md` (read-only if present)
- Cache paths (written by reviewers, stored under `artifact/`):
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
- Read `discovery_path` when present and valid.
- Treat the finalized code/test steps as the source of truth.
- Modify existing I#/T# step files only when the initial code-documentation pass or reviewer findings target them.

## 2. Apply an initial code-documentation pass
- Scan I#/T# diffs for missing API docs, parameter/return docs, `# Errors` sections, and inline readability comments required by the documentation and errors rules.
- For non-trivial function-body changes, apply the imported inline readability-comment rules inside the affected planned code diff; place comments at logical steps, not as generic notes.
- Put documentation changes in the relevant step diff or snippet. A generic note such as `update docs` does not satisfy the rule files.
- Preserve the step's existing action, intent, and approximate line labels; add or adjust only the minimal affected diff hunks.

## 3. Run the code-documentation review loop
- Write and maintain `## Delta` in `handoff_path`. Record each I# and T# step as a Delta entry with `Status:`, `Touched:`, and `Why:` fields. Recompute `## Delta` after every material revision.
- Mark unchanged items as `Unchanged` with `Why: no content change`.
- Treat `handoff_path` as the shared ledger for reviewer findings, statuses, and arbitration decisions. Reviewers maintain their own cache files; do not copy cache state into the handoff.
- Run these independent shared code-doc reviewers in parallel on the first pass:
  - `_plan/finalize-codedoc-reviewers/docs-and-readability-cached`
  - `_plan/finalize-codedoc-reviewers/errors-cached`
- Pass each reviewer only run data: `plan_path`, `handoff_path`, exact `step_paths`, `cache_path`, changed ids/paths, trigger flags, and short `user_notes`.
  - For docs-and-readability: `cache_path: artifact/<artifact_base>.review-codedoc-docs-readability.md`
  - For errors: `cache_path: artifact/<artifact_base>.review-codedoc-errors.md`
- Update the `## Review Ledger` in `handoff_path`: assign IDs to new findings, preserve existing IDs when the underlying issue is unchanged, mark resolved issues RESOLVED, defer non-blocking issues DEFERRED.
- Apply domain ownership: CDOC and CREAD → docs-and-readability reviewer; CERR → errors reviewer. CDOC owns required API docs and inline readability comments in planned code diffs. Arbitrate cross-domain conflicts.
- Apply all BLOCKING fixes before advisories. Resolve CDOC/CERR before CREAD when fixes conflict. Record or defer advisories when no blockers remain.
- Apply reviewer diffs to existing I# and T# step files only. Append one line to `## Revision History`.
- Re-run only reviewers whose owned domain or touched step changed after a material revision; rerun both reviewers when a fix changes both code docs and error docs. Do not rerun unrelated domains.
- Loop until no BLOCKING findings remain or 10 iterations.
  No blocking: SUCCESS with recorded/deferred advisories. At cap: FAIL if BLOCKING, SUCCESS with risks if only ADVISORY.
- Validate each reviewer response against the review block shape: starts with `# REVIEW`, contains `Decision: PASS | ADVISORY | BLOCKING`, contains `## Findings` and `## Verified` headings. Treat malformed responses as BLOCKING with a synthetic finding.
- Before `Status: SUCCESS`:
  - Audit errors with `_plan/finalize-codedoc-reviewers/errors-cacheless` when public API/error-docs changed.
  - Audit docs-and-readability with `_plan/finalize-codedoc-reviewers/docs-and-readability-cacheless` when doc comments changed.
  - Ignore caches and Delta shortcuts.
  - Return all current findings.
  - If BLOCKING: fix, recompute Delta, rerun touched reviewers, then re-audit.

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
