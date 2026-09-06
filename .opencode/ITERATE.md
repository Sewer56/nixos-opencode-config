# Iterate guide

`/iterate/edit` changes or verifies instructions and related OpenCode files.
Use `/draft` and `/implement` for product code.
Give exact paths or describe observable behavior and its command/role.

```text
/iterate/edit Move config/command/write/pr.md and preserve routing.
/iterate/edit Verify implementation reviewers cannot edit code; change nothing.
```

## Lifecycle

1. Inspect targets, consumers, instructions, and dependent checks.
2. Lock exact actions in a behavioral contract.
3. Delegate to one editor, then stage permitted target changes.
4. Run deterministic checks before independent review and finding verification.
5. Allow at most two repair turns before final checks and result.

Editor chooses routine in-scope details without questions.
Precedence resolves apparent conflicts, not real authority conflicts.
Only unresolved material choices or incompatible target edits need questions.

Current target edits are input; unrelated index/worktree state is preserved.
Changed targets are reread; compatible edits need no lock question.
Only orchestrator stages; staging-only issues do not block writing.

## Contract

`contract.md` records base commit, actions, behavior, non-goals, and lenses.
Actions are `CREATE`, `UPDATE`, `DELETE`, `MOVE`, or `VERIFY`.
Consumers provide context, not write permission.

Assertions needing changes must be `UPDATE` before scope freezes.
`VERIFY` is no-edit; pure moves preserve bytes/mode unless contracted.
Frozen contract defects are `INCOMPLETE`, never permission to expand or ask.

Updates preserve decision boundaries at equal or smaller token count.
Orchestrator records reproducible old/new counts.

## Continuation

Orchestrator saves returned editor `task_id` in run-local `editor-task.md`.
Repairs and same-run continuation reuse it only with unchanged authority.
Revalidate request, contract, and target state before resuming.

The ID is a tool argument, never another `<editor-inputs>` prompt field.
Changed authority requires fresh preflight and a new task.

Stale or unavailable identity requires a recorded fallback before a new task.
Never silently reuse identity across runs.

## Checks and review

Orchestrator inspects actual staged actions and runs:

```bash
git diff --cached --name-status --find-renames HEAD -- <target paths>
git diff --cached --check -- <target paths>
python3 scripts/validate-opencode-config.py --repo-root .
python3 -m unittest discover -s tests -p 'test_*.py'
```

Workflow tests apply to control-file, test, and validator changes.
Static regression checks are not live agent execution.
Validator checks syntax, imports, routing, permissions, and documentation links.

Missing required evidence is `INCOMPLETE`, not PASS.
Use `nix develop` if declared Python dependencies are missing.

Independent review applies required lenses to the staged diff.
Separate verifier attempts to refute candidates only when the review reports findings; it is skipped when there are none.

Only deterministic failures and verified `TARGET` blockers reach editor.
Advisories remain visible without automatic repair.

Self-edits require workflow tests plus architecture and adversarial review.
Each repair reruns checks, affected reviews, and candidate verification.

## Artifacts and outcomes

```text
request.md, contract.md       intent and frozen authority
preflight.md, editor-task.md  checks and run-bound identity
validation.md, tests.md       deterministic results
review.md, verdict.md         review and conditional verification
result.md                    status, actions, checks, reviews, missing evidence
```

- `SUCCESS`: permitted exact actions and all required gates pass.
- `NEEDS_INPUT`: unresolved material choice needs a decision.
- `INCOMPLETE`: contract defect or missing required evidence.
- `FAIL`: target blocker survives two repairs or authority integrity fails.

Restore only accidental workflow edits, never pre-existing changes.
Do not widen a frozen contract to repair an out-of-scope defect.
