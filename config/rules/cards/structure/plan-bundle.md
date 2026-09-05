### Plan bundle boundary
- `plan_path: None` keeps standalone authority and real handoff.
- Use Git-root `PROMPT-PLAN-[[slug]].draft.md` as `plan_path`.
- Members stay in `artifact/plan/[[plan]]/`.
- `[[plan]]`: root basename minus `.draft.md`.
- Root links define the bundle.
- Require `contract.md` and numeric cohorts.
- `handoff_path`/`cohort_path` alias declared contract/cohort.
- Preserve the bundle; scope changes need `/draft` and approval.
- Request sets intent; approved plan, not evidence, sets behavior.

Missing/conflicting required plan authority:
- Draft review/verification: `BLOCKED`.
- Implementation review/finding verification: `INCOMPLETE`.
- Writers and implementation parent: `NEEDS_INPUT`.

### Role-specific reads
- Draft readiness/review/verification: full bundle.
- Explorer: discovery authority.
- Implementation parent: root/member metadata, not member contents.
- Writers/cohort reviewers: root routing, contract, assigned cohort.
- Read only relevant declared references.
- Final reviewers: full acceptance and cross-cohort composition authority.
- Repair/finding verification: supplied-issue authority.

Parent gates may read Git, validation, reviews, verdicts, completion.

### Implementation resume
- Preserve work/artifacts; never delete or auto-unstage.
- Adopt only authorized partial work; validate/review fresh diffs.
- Keep original baselines/consumed turns.
- Unclear ownership/material facts need `NEEDS_INPUT` before writing.
