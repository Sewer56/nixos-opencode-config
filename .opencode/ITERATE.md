# Iterate guide

`/iterate/edit` changes or verifies instructions and related OpenCode files.
Use `/draft` and `/implement` for product code.
Give exact paths or describe observable behavior and its command/role.

```text
/iterate/edit Move config/command/write/pr.md and preserve routing.
/iterate/edit Verify implementation reviewers cannot edit code; change nothing.
```

## Lifecycle

1. Discuss goal, constraints, design, success and a small task outline.
2. Use bounded read-only discovery and focused questions until agreement.
3. Require explicit design agreement and authorization before any documents.
   Reuse earlier agreement for unchanged scope; invocation or thanks is not it.
   Before agreement, create no request, contract, run record or local exclude.
4. Inspect targets/consumers/checks and freeze exact actions in a contract.
5. Delegate to one editor, then stage permitted target changes.
6. Run deterministic checks before independent review and finding verification.
7. Allow at most two repair turns before final checks and result.

Editor chooses mechanics; material choices/incompatible edits need input.

Current target edits are input; unrelated index/worktree state is preserved.
Only orchestrator stages; staging-only issues do not block writing.

## Contract

`contract.md` records base commit, actions, behavior, non-goals, and lenses.
Actions are `CREATE`, `UPDATE`, `DELETE`, `MOVE`, or `VERIFY`.

Assertions needing changes must be `UPDATE` before scope freezes.
`VERIFY` is no-edit; pure moves preserve bytes/mode unless contracted.
Frozen contract defects are `INCOMPLETE`, never permission to expand or ask.

Updates preserve decision boundaries at equal or smaller token count.
Orchestrator records raw cl100k_base old/new counts and expanded agent bodies.
Expanded counts include imports so deduplication cannot hide prompt growth.

## Continuation

Orchestrator saves returned editor `task_id` in run-local `editor-task.md`.
Repairs and same-run continuation reuse it only with unchanged authority.
Revalidate request, contract, and target state before resuming.

The ID is a tool argument, not a prompt field; never reuse it across runs.
Changed authority needs fresh preflight/task; stale IDs need recorded fallback.

## Checks and review

Orchestrator inspects actual staged actions and runs:

```bash
git diff --cached --name-status --find-renames HEAD -- <target paths>
git diff --cached --check -- <target paths>
python3 scripts/validate-opencode-config.py --repo-root .
bash scripts/check-workflows.sh
```

The shell smoke exercises routing/pair/cycle/path safety in temporary fixtures.
Static/scenario checks are not live agent execution or usability evidence.
Validator checks syntax, imports, routing, permissions, and documentation links.

Missing required evidence is `INCOMPLETE`, not PASS.
Use `nix develop` if declared Python dependencies are missing.

Independent review applies required lenses to the staged diff.
Verifier refutes candidate findings only; clean reviews skip it.

Only deterministic failures and verified `TARGET` findings reach editor.
Apply feasible advisories within frozen scope and budget.
Explain skipped advisories; they never block success.

Self-edits require shell smoke plus architecture and adversarial review.

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

Repairs rerun checks/reviews without widening scope or losing existing edits.
