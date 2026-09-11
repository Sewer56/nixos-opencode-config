{{ file="./rules/review/contract.md" }}

`_review/verifier` alone judges local candidate accuracy and repair eligibility.

Before verdicts, orchestrators only check routing metadata.
Never investigate or filter findings before verifier disposition.
Route every reported candidate, with its justification, to `_review/verifier`.

Group all domains sharing input identity into one `boundary_id` partition.
Identity: authority, review scope/exclusions, scope/boundary/base/head/paths.

Pass shared inputs, domains/candidate IDs and paths, boundary_id and round.
Send only candidate-bearing partitions, without siblings.
Use one verifier call per partition, not per domain or finding.

Use unique verdict paths/rounds; await every disposition before repair.
Missing/mismatched results mean INCOMPLETE.

Deduplicate repairs within unchanged budgets.
Return all verdict paths; retain identities/evidence for resume/handoff.
Obsolete domains or input schemas require fresh review, not relabeled evidence.

Prior reports lacking justified-v1 cases cannot authorize repairs.

### Accepted repairs

Prioritize deterministic failures and verifier-accepted blockers.
Apply feasible accepted advisories.

Check source to implement accepted repairs, not to re-adjudicate findings.
Reuse verified edits or follow verified requirements.
Never use rejected or unresolved corrections.

Send contradictory repair evidence back to the assigned verifier.
Delegate reverification of material departures from verified corrections.

Explain nonblocking skips; preserve scope, decisions and budgets.
Recheck/review edits.
