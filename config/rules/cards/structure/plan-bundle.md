### Plan bundle boundary
- With `plan_path: None`, retain standalone authority and the real handoff.
- Use Git-root `PROMPT-PLAN-[[slug]].draft.md` as `plan_path`.
- Members stay in `artifact/plan/[[plan]]/`.
- `[[plan]]` is the root basename without `.draft.md`.
- Root links define the bundle, including shared documents and references.
- Require `contract.md` and numeric cohorts like `01-models.md`, `02-config.md`.
- `handoff_path` and `cohort_path` alias the declared contract and cohort.
- Implementation preserves the bundle; scope changes need `/draft` and approval.
- Request sets intent; approved plan sets behavior, not repository evidence.

Stop for missing or conflicting required plan authority:
- Draft review/verification: `BLOCKED`.
- Implementation review/finding verification: `INCOMPLETE`.
- Writers and implementation parent: `NEEDS_INPUT`.

### Role-specific reads
- Draft readiness/review/verification reads the entire bundle.
- Explorer reads discovery-relevant authority.
- Implementation parent reads root and member metadata, not member contents.
- Writers/cohort reviewers read root routing, contract, and assigned cohort.
- Include only relevant declared references in scoped reads.
- Final reviewers read full acceptance and cross-cohort composition authority.
- Repair/finding verification reads authority relevant to supplied issues.

Parent gates may read Git, validation, reviews, verdicts, and completion.

### Implementation resume
- Preserve prior work/artifacts without deletion or automatic unstaging.
- Adopt only authorized partial changes; validate/review current diffs afresh.
- Keep original baselines and consumed turns.
- Unclear ownership/material facts need `NEEDS_INPUT` before writing.
