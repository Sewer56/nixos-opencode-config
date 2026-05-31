---
description: "Implement a plan from conversation context with automated review loop and code-quality cleanup"
agent: _implement/freeform
---

Implement a plan from conversation context with an automated review loop and code-quality cleanup.

# Inputs
- Plan exists in prior conversation messages.
- Derive `slug` from request context as a 2–3 word identifier and `artifact_base = PROMPT-PLAN-<slug>`.
- Use `plan_path = <cwd>/<artifact_base>.draft.md`.

# Extra Instructions
$ARGUMENTS

# Workflow

## 1. Preflight
- Stop with `FAIL` when no implementable request or prior plan exists.

## 2. Write plan
- Extract the original user request and plan steps from conversation context.
- Write `plan_path` with:
  - `## Original Request`: the user's request verbatim or summarized.
  - `## Plan`: ordered implementation steps from conversation context.
- Do not rewrite unrelated `PROMPT-PLAN-*` artifacts.

## 3. Implement
- Follow plan steps in order.
- Run formatter, linter, build, and tests after each cohesive change group.
- Iterate until all checks pass clean.

## 4. Review
- Spawn `_implement/freeform/reviewer` with:
  - `plan_path`: absolute path to the plan file.
  - `changed_paths`: comma-separated list of changed files.
  - `notes`: 0-2 current-run facts or `None`.
- Wait for the response.

## 5. Review loop
- Parse `Decision:` and `## Findings` from the inline `# REVIEW` block.
- If the response is malformed or missing the block, retry.
- If any findings remain, fix them and re-run reviewer with updated run data.
- Repeat until `Decision: PASS` or 5 iterations.
- At cap with findings remaining, return `FAIL`.
- Before proceeding, run one final audit with `_implement/freeform/reviewer` and updated run data.
- If final audit has BLOCKING findings, fix, rerun touched work, and re-audit.

## 6. Cleanup review phase
- Derive `changed_source_files`: filter `changed_paths` to source code files (exclude docs, config, assets).
- Derive `changed_doc_files`: filter `changed_paths` to user-facing documentation files (`*.md`, `docs/**`, `README*`).
- Spawn in parallel:
  - `_implement/reviewers/code-docs` with `changed_paths=changed_source_files` and short notes.
  - `_implement/reviewers/errors` with `changed_paths=changed_source_files` and short notes.
  - `_implement/reviewers/placement` with `changed_paths=changed_source_files` and short notes.
- If `changed_doc_files` is non-empty, also spawn in parallel:
  - `_implement/reviewers/user-docs` with `changed_paths=changed_doc_files` and short notes.
  - `_implement/reviewers/polish` with `changed_paths=changed_doc_files` and short notes.
- Pass only `changed_paths` and short `notes` to each reviewer. Do not duplicate role text, process steps, or output schema.
- Parse `Decision:` and `## Findings` from each inline `# REVIEW` block.
- If any response is missing or malformed, retry that reviewer.
- If any reviewer returns BLOCKING findings, apply fixes, then rerun only reviewers whose domain overlaps with the fix.
- Loop until all reviewers return PASS with 0 findings or 3 cleanup iterations.
- At cap with BLOCKING remaining, record remaining findings and continue.

## 7. Report
- Return final status. No auto-commit.

# Output
Return exactly:

```text
Status: SUCCESS | INCOMPLETE | FAIL
Plan Path: <absolute path | N/A>
Iterations: <n>
Cleanup Iterations: <n>
Summary: <one-line summary>
```
