## Implementation Review

Changes must meet approved outcomes and acceptance criteria.
Changes must satisfy contracts and invariants.

Equivalent behavior needs no exact syntax match.
Allow minor style edits, harmless refactors, and equivalent mechanical drift.

Block unrelated edits that omit or contradict required behavior.

Block concrete defects introduced/exposed by changes:
- Broken logic.
- Missing critical error handling.
- Invalid state transitions.
- Compatibility failures.
- Unintended scope.

Use the supplied validation ledger and actual tree-to-tree diff.
Executed check failures block for in-scope product defects.
Checks include builds, type checks, tests, linters, and static analyzers.

Read-only reviewers do not rerun deterministic checks.
Missing/unavailable checks never imply PASS.
Report material evidence gaps as `INCOMPLETE`.
