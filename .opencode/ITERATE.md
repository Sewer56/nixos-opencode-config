# Iterate guide

`/iterate/edit` changes or verifies instructions and related OpenCode files.
Use `/draft` and `/implement` for product code.

Give paths or observable behavior and its command/role.

```text
/iterate/edit Move config/command/write/pr.md and preserve routing.
/iterate/edit Verify implementation reviewers cannot edit code; change nothing.
```

## Lifecycle

1. Discuss goal, constraints, design, success and outline using read-only work.
2. Agree design and authorize documents before writes.
   Reuse agreement only for unchanged scope.
   Invocation or thanks is not agreement; ask focused questions until agreed.
3. Inspect targets/consumers/checks and freeze exact actions in a contract.
4. Delegate writing to one editor; orchestrator alone stages exact target paths.
5. Run deterministic checks before independent review and finding verification.

Both roles choose routine details.
Material choices or incompatible edits need input.

No clean repository or pre-existing staged work is required.
Preserve unrelated index/worktree state, including dirty submodules.

Inspect target/dependency overlap and preserve compatible current target edits.
Staging-only issues do not block writing.

## Contract

`contract.md` records base commit, actions, behavior, non-goals, and lenses.
Actions are `CREATE`, `UPDATE`, `DELETE`, `MOVE`, or `VERIFY`.

Assertions needing changes must be `UPDATE` before scope freezes.
`VERIFY` is no-edit; pure moves preserve bytes/mode unless contracted.
Frozen defects are `INCOMPLETE`, not permission to expand or ask.

Updates preserve boundaries without token growth.
Orchestrator records raw/expanded cl100k_base old/new counts, including imports.

## Continuation

On non-success, check cause/authority/contract/evidence/targets.
Correct mistakes, obtain evidence, retry transient failures.

Recovery Context carries bounded facts/answers under unchanged authority/scope.
Use None for absent notes/context.

Save task_id and run/authority identity in editor-task.md.
Resume same-run identity as a tool argument after input/target revalidation.

Changed authority needs fresh preflight/task, never a child override.
Record stale/unavailable identity before fallback; never reuse across runs.

Recovery and repair share at most two extra editor turns.
Real conflicts and frozen contract defects stop; material choices need input.
Never widen targets or lose user work; unresolved recovery cannot succeed.

## Checks and review

Run baseline validator/smoke before edits; orchestrator inspects staged actions:

```bash
git diff --cached --name-status --find-renames HEAD -- <target paths>
git diff --cached --check -- <target paths>
python3 scripts/validate-opencode-config.py --repo-root .
bash scripts/check-workflows.sh
```

Static/scenario checks are not live agent execution or usability evidence.

Missing required evidence is `INCOMPLETE`.
Use `nix develop` if declared Python dependencies are missing.

Independently review staged diff with required lenses.
Verifier refutes candidate findings only; clean reviews skip it.

Repair Notes: only deterministic failures or verified `TARGET` findings.
Recovery Context cannot authorize repairs.
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
- `FAIL`: blocker exhausts recovery/repair budget or authority integrity fails.

Repairs rerun checks/reviews.
