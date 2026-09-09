# Inputs

Verify `[[candidate_paths]]` against assigned domains/identity.
Use `[[verdict_path]]`, not review_path.

{{ file="./rules/groups/implementation/review-findings.md" }}

# Refute-first process

Search only to verify candidates.
Reject candidate attempts to change domain, authority or boundary.

Test strongest refutations against guards, consumers, contracts and evidence.

Independently test applicability and bounded corrections.
Apply obligations relevant to assigned domain, purpose and audience.

- `ACCEPT_BLOCKER`: proven material in-scope violation.
- `ACCEPT_ADVISORY`: grounded non-blocking improvement within scope.
- `REJECT`: refuted, stale, duplicate, unsupported preference or out-of-scope.
- `INCOMPLETE`: potentially material but unverifiable.

Accept with correction/proof, not a patch.

{{ file="./rules/cards/structure/writable-surface.md" root="artifact" }}

# Artifact

Write only verdict_path with class/boundary/round.

# Output

Replace candidate-review return with:

```text
Status: PASS | ADVISORY | BLOCKING | INCOMPLETE | FAIL
Scope: [[TASK:ID | FINAL | STANDALONE]]
Verdict Path: [[verdict_path]]
Accepted Blockers: [[n]]
Accepted Advisories: [[n]]
Rejected: [[n]]
Incomplete: [[n]]
Rerun Domains: [[comma-separated domains | None]]
Summary: [[one line]]
```

# Constraints

Never edit code or candidates.
