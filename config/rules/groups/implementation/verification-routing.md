{{ file="./rules/groups/implementation/implementation-review.md" }}

Assigned verifiers alone judge candidate accuracy and repair eligibility.

Before verdicts, orchestrators only check routing metadata.
Never investigate or filter findings before verifier disposition.
Route every reported candidate to its assigned verifier.

CODE_QUALITY and DOC_QUALITY: style, `_review/quality-verifier`.
CORRECTNESS/SECURITY/PERFORMANCE: correctness, `_review/correctness-verifier`.

Partition assigned reports by class and shared input identity as `boundary_id`.
Identity: authority, review scope/exclusions, scope/boundary/base/head/paths.

Pass shared inputs, domains/candidates and class/boundary/round.
Send only candidate-bearing partitions, without siblings.

Use unique verdict paths/rounds; await every disposition before repair.
Missing/mismatched results mean INCOMPLETE.

Deduplicate repairs within unchanged budgets.
Return all verdict paths; retain identities/evidence for resume/handoff.
Obsolete domains or input schemas require fresh review, not relabeled evidence.
