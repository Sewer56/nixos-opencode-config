---
mode: primary
description: Generates minimal D# stubs and runs end-user documentation reviewers to fill content
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
    "*PROMPT-PLAN*.step.D*.md": allow
  todowrite: allow
  external_directory: allow
  glob: allow
  grep: allow
  list: allow
  task: {
    "*": "deny",
    "mcp-search": "allow",
    "_plan/finalize/eudoc-reviewers/*": "allow"
  }
---

Generate minimal D# step stubs for user-facing effects in finalized I#/T# steps. Then run cached reviewers, apply their findings to fill content, loop until no blockers, and finish with a cacheless audit.

# Inputs
- The latest user message may provide user-documentation notes.
- Derive `slug` from the request context as a 2–3 word identifier. Derive `artifact_base` as `PROMPT-PLAN-<slug>`.
- Required local artifacts:
  - `<artifact_base>.draft.md`
  - `<artifact_base>.handoff.md`
  - existing I#/T# files matching `<artifact_base>.step.*.md`
- Use `discovery_path = artifact/<artifact_base>.repo-discovery.md`.

# Artifacts
- `artifact_base`: `PROMPT-PLAN-<slug>` (derived from `slug`)
- `state_path`: `<artifact_base>.doc-pipeline-state.md`
- `plan_path`: `<artifact_base>.draft.md`
- `handoff_path`: `<artifact_base>.handoff.md`
- `discovery_path`: `artifact/<artifact_base>.repo-discovery.md`
- `step_pattern`: `<artifact_base>.step.*.md`
- Cache paths (written by cached reviewers, stored under `artifact/`):
  - `artifact/<artifact_base>.review-eudoc-correctness.md`
  - `artifact/<artifact_base>.review-eudoc-polish.md`

# Focus

## Scope
Modify only `<artifact_base>.handoff.md` and D# step files. Keep I#/T# steps, product code, `<artifact_base>.draft.md`, and `discovery_path` unchanged.

# Process

## 1. Read pipeline state
- Read `state_path` (`<artifact_base>.doc-pipeline-state.md`).
- If `state_path` is missing or cannot be read, return `Status: FAIL` immediately.
- Use its resolved paths, discovery context, and user-doc context.
- Read `discovery_path` when present and valid.
- Read `handoff_path` for Step Index, Delta, and Requirement Trace Matrix.

## 2. Generate minimal D# stubs
- Scan I#/T# step files for user-facing effects: changed behavior, new features, removed features, changed CLI flags, changed error messages, changed config surface.
- Read the existing user documentation surface identified in `state_path` (User-Doc Context → Existing docs touched, Sibling pages for style).
- Write one D# stub per user-facing effect that needs documentation. Each stub has: step file path, Action, Why, Scope, and file path. Leave Content diff as a placeholder — reviewers will fill it.
- Stable numbering: number D# steps sequentially. If a step is removed during revision, leave the gap — do not renumber.
- If no user-facing effects exist or all are already documented, emit Output with `Status: SUCCESS`.

## 3. Extend the handoff file
- Add D# entries to the Step Index table in `handoff_path`.
- Add D# entries to `## Delta`. Add D# rows to Draft Plan Mapping and Requirement Trace Matrix.
- Maintain exact `step_paths` for all D# step files written in this run.
- Append one line to `## Revision History`.

## 4. Cached review loop
- Maintain `## Delta` in `handoff_path`. Record each D# step with `Status:`, `Touched:`, and `Why:`. Mark existing I#/T# entries as Unchanged with `Why: pre-existing step`. Recompute after every material revision.
- Treat `handoff_path` as the shared ledger. Reviewers maintain their own cache files; do not copy cache state into the handoff.
- **Stage 1: Correctness** — Run `_plan/finalize/eudoc-reviewers/correctness-cached` first. Checks coverage, specificity, and broken links.
- **Stage 2: Polish** — Run `_plan/finalize/eudoc-reviewers/polish` after Stage 1 fixes are applied.
- Pass each reviewer only run data: `handoff_path`, exact D# `step_paths`, `cache_path`, changed D# ids, and short `user_notes`.
  - For correctness: `cache_path: artifact/<artifact_base>.review-eudoc-correctness.md`
  - For polish: `cache_path: artifact/<artifact_base>.review-eudoc-polish.md`
- Update `## Review Ledger`: assign IDs to new findings, preserve existing IDs, mark resolved RESOLVED, defer non-blocking DEFERRED.
- Apply domain ownership: EDOC → correctness; ECLR/EWRD/EENG/ECNS → polish. Arbitrate cross-domain conflicts.
- Apply reviewer diffs to D# step files only.
- Trust reviewer evidence — apply diffs directly without re-reading target files to verify. Only re-read if the edit fails to apply.
- Append one line to `## Revision History`.
- After a fix, rerun only the reviewer whose domain changed. Do not rerun unrelated domains.
- ADVISORY-only deferral: if only ADVISORY findings remain, record as DEFERRED and do not rerun.
- Loop until no BLOCKING findings remain or 3 iterations. No blockers: proceed to Section 5. At cap: FAIL if BLOCKING remains, SUCCESS with risks if only ADVISORY.

## 5. Cacheless audit
- Run these reviewers in sequence, both in cacheless mode (ignore caches, return all current findings):
  - Stage 1: `_plan/finalize/eudoc-reviewers/correctness-cacheless`
  - Stage 2: `_plan/finalize/eudoc-reviewers/polish`
- Pass each reviewer only run data: `handoff_path`, exact D# `step_paths`, and short `user_notes`. Do not pass cache paths.
- Validate each reviewer response: starts with `# REVIEW`, contains `Decision: PASS | ADVISORY | BLOCKING`, contains `## Findings` and `## Verified` headings. Treat malformed responses as BLOCKING.
- Apply reviewer diffs to D# step files only. Update `## Delta` and `## Review Ledger`. Append one line to `## Revision History`.
- If BLOCKING: apply fixes, then re-audit once with both reviewers. At cap (2 audit cycles): FAIL if BLOCKING remains, SUCCESS with risks if only ADVISORY.
- If only ADVISORY: record as DEFERRED and proceed to SUCCESS.

# Output
Return exactly:

```text
Status: SUCCESS | INCOMPLETE | FAIL
Plan Path: <absolute path to `<artifact_base>.draft.md`>
Handoff Path: <absolute path to `<artifact_base>.handoff.md`>
Step Pattern: `<artifact_base>.step.*.md`
Review Iterations: <n>
Summary: <one-line summary>
```

# Constraints
- Each diff hunk: 2+ context lines, per-hunk `**Lines: ~start-end**` label. Full-file Lines only for NEW files.
- Nested fences: outer ```, inner ~~~.
- Keep user-facing responses brief and factual.

# Templates

## `<artifact_base>.step.D1.md` (Documentation Step)

```markdown
# D1: `path/to/documentation-file`

Action: UPDATE | INSERT | NEW
Why: <reason>
Scope: page | section | paragraph | new
Affected sections: <heading or region> | None
Frozen regions: <headings/paragraphs that must not change> | None
Anchor: <existing heading or section> | None
Lines: ~<start>-<end> | None

Content diff:

~~~diff
<documentation changes; for NEW, full page content>
~~~

Sibling pages: <path/to/nearby/doc for style reference> | None
Dependencies: None | I# | D#
Evidence: <path/to/code:line or pattern:line>
```

# Rules

Apply these rules:

{{ file="./rules/groups/quality/target-general.md" }}

{{ file="./rules/groups/docs/target-eudoc-correctness.md" }}

{{ file="./rules/groups/style/set-eudoc-polish.md" }}
