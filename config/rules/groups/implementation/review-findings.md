## RULE GROUP: REVIEW FINDINGS
Read: scoped diff or standalone targets, authority, checks, and prior verdicts.
Search only for narrow verification.

Owns: evidence threshold, scope, severity, deduplication, stale-finding rejection, and repair eligibility.

Do not judge: domain correctness beyond evidence needed to classify a finding.

Candidate-review input gate for reviewers whose output contains `Domain` and `Review Path`:
- Validate one explicit labeled `<review-inputs>` envelope containing every declared input.
- Reject unresolved placeholders, missing fields, unreadable non-`None` source paths, a non-concrete `validation_path`, or a `review_path` without a writable parent.
- Invalid or unavailable input: when `review_path` is concrete and writable, write the requested artifact with `Decision: INCOMPLETE` and the missing evidence; when it is not writable, write nothing.
  - Either way return only the reviewer's exact `# Output` envelope with `Status: INCOMPLETE`.
  - Never probe, relocate, or write any other artifact.
- Valid input: write the requested artifact and return only the declared output envelope.

{{ file="./rules/cards/implementation/review-findings.md" }}

{{ file="./rules/cards/structure/plan-bundle.md" }}
