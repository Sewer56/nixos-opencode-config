## Implementation Review

Changes must satisfy approved outcomes, acceptance criteria, contracts, and invariants.
Exact syntax is unnecessary when behavior is equivalent.

Block unrelated edits that omit or contradict required behavior.

Block concrete broken logic, missing critical error handling, invalid state transitions, compatibility failures, and unintended scope introduced/exposed by the change.

Allow minor style differences, harmless refactors, and behavior-equivalent mechanical drift.

### Functional evidence
Use the supplied validation ledger and actual tree-to-tree diff.

Executed failing builds, type checks, tests, linters, and static analyzers are blocking when they identify an in-scope product defect.

Read-only reviewers do not rerun deterministic checks.
Never infer PASS from a missing/unavailable check; report material evidence gaps as `INCOMPLETE`.
