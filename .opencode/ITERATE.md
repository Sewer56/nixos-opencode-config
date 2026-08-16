# Iterate guide

`/iterate/edit` creates, updates, moves, deletes, or verifies LLM instructions and related OpenCode agents, commands, rules, skills, validation, tests, and human docs.

Use it for instruction artifacts. Use `/draft` and `/implement` for product code.

## Examples

```text
/iterate/edit Update config/agent/codebase-explorer.md to report direct consumers.
/iterate/edit Move config/command/write/pr.md and preserve routing.
/iterate/edit Delete an obsolete agent and every active route to it.
/iterate/edit Verify implementation reviewers cannot edit code; change nothing.
/iterate/edit Simplify the iterate workflow without weakening review.
```

Give exact paths when known. Otherwise name observable behavior and current command/role. Workflow asks at most one material question before editing.

## Lifecycle

```text
request
  -> discover exact targets and direct consumers
  -> write compact behavioral contract
  -> one editor
  -> stage exact target paths
  -> deterministic checks
  -> focused review when behavior or risk requires it
  -> refute-first finding verification
  -> at most two repairs
  -> final result
```

Workflow assumes no concurrent writer. It edits current target contents, stages only target paths, and ignores unrelated Git state.

## Contract

Run-local `contract.md` names:

- base commit;
- exact `CREATE`, `UPDATE`, `DELETE`, `MOVE`, or `VERIFY` actions;
- observable required behavior;
- behavior to preserve and non-goals;
- required behavior, architecture, and adversarial review lenses.

Consumers define context, not edit permission. `VERIFY` means no change. Pure moves preserve bytes/mode unless contract says otherwise.

## Deterministic checks

Orchestrator stages only contracted paths, then checks:

```bash
git diff --cached --name-status --find-renames HEAD -- <target paths>
git diff --cached --check -- <target paths>
python3 scripts/validate-opencode-config.py --repo-root .
python3 -m unittest discover -s tests -p 'test_*.py'
```

Implementation tests run when workflow/validator changes. Whole-config validator covers parsing, frontmatter, imports, routes, reachability, task depth, permissions, syntax, documentation links, and required global options.

## Review

One reviewer applies only required behavior, architecture, and adversarial lenses to staged diff. Separate verifier attempts to refute candidates only when the review reports findings; it is skipped when there are none. Only verified target blockers reach editor. Advisories remain visible but are not automatic repairs.

Self-edits rerun whole-config validation and implementation workflow tests, then receive architecture and adversarial review.

## Artifacts

```text
request.md      original request
contract.md     concise edit authority
preflight.md    optional pre-edit checks for self-change
validation.md   whole-config validation result
tests.md        implementation workflow test result when required
review.md       conditional candidates
verdict.md      conditional verified findings
result.md       concise final outcome
```

After interruption, inspect staged diff and artifacts; start fresh when authority is uncertain.

## Outcomes

- `SUCCESS`: exact staged actions, checks, required review, and verification pass.
- `NEEDS_INPUT`: material choice needs human decision.
- `INCOMPLETE`: required live/evaluation evidence unavailable or contract defect discovered after editing.
- `FAIL`: proven target defect remains after two repairs or authority integrity failed.

## Troubleshooting

### Out-of-contract change

Restore only accidental workflow edit manually or start fresh with correct explicit target. Never widen contract after editing.

### Validation dependency missing

Use `nix develop` for declared Python dependencies. Missing required environment is `INCOMPLETE`, not PASS.

## Active files

| File | Role |
|---|---|
| [`edit.md`](agent/_iterate/edit.md) | Orchestrator, contract, staging, checks, repair limit. |
| [`editor.md`](agent/_iterate/editor.md) | Sole instruction writer. |
| [`review.md`](agent/_iterate/review.md) | Focused independent review. |
| [`verifier.md`](agent/_iterate/verifier.md) | Refute-first finding gate. |
| [`instruction-authoring.md`](rules/instruction-authoring.md) | Runtime authoring standard for future commands. |

See [instruction architecture rationale](../EXPLAINER.md#instruction-authoring-and-iterate).
