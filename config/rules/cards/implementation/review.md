### Implementation fidelity
Changes must satisfy the approved outcome, acceptance criteria, contracts, and invariants. Exact syntax is not required when behavior is equivalent. Block unrelated edits that omit or contradict required behavior.

### No severe regression
Block concrete broken logic, missing critical error handling, invalid state transitions, compatibility failures, and unintended scope introduced or exposed by the change.

Allow minor style differences, harmless refactors, and behavior-equivalent mechanical drift.

### Functional evidence
Use the supplied validation ledger and actual tree-to-tree diff.

Treat an executed failing build, type check, test, linter, or static analyzer as blocking when it identifies a product defect in scope.

Do not rerun deterministic checks from a read-only review agent and do not infer PASS from a missing or unavailable check; report the evidence gap as `INCOMPLETE` when material.
