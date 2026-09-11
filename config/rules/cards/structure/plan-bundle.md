### Plan authority
Use Git-root `PROMPT-PLAN-[[slug]].draft.md` as `plan_path`.
Members: `artifact/plan/[[root basename without .draft.md]]/`.
Root and numeric human briefs own product decisions and outcomes.

Root links `execution.md`; each `NN-name.md` links `NN-name.exec.md`.
Execution translates human authority without hidden decisions.
Use only the routing syntax checked by `config/scripts/plan-bundle.py`.

Reject combined legacy plans and plan contracts/aliases; never auto-convert.
Standalone handoffs and agreed iterate scope remain separate valid authority.

Evidence and runtime `review/` are not source members or writable plans.
Missing/conflicting authority stops writers with `NEEDS_INPUT`.
Review returns `INCOMPLETE`; draft review returns `BLOCKED`.

### Reads and resume
Draft readiness/review/verifier read the whole bundle.
Parent reads root, shared execution/routing and evidence, not sibling execs.

Workers/reviewers read root, assigned brief/exec and relevant shared refs.
Repairs/verdicts load issue-relevant authority.

Preserve source; changed scope needs `/draft` and approval.
Preserve work, original baselines and consumed budgets on resume.

Adopt only authorized partial work; check/review fresh diffs.
Unclear ownership needs input; never delete or auto-unstage prior work.
