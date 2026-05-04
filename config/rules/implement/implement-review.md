### Objectives met
Each implementation step (or inline request goal) must have corresponding code changes that satisfy the stated objective.

Bad: handoff step adds validation but `git diff` has no validation change.
Good: changed files implement every planned step, or notes explain why a step became unnecessary.

### Implementation fidelity
Changes should match the described code shape and anchors without requiring exact textual adherence when behavior is equivalent.

Bad: implementation edits unrelated files and omits planned anchor behavior.
Good: implementation uses equivalent helper placement while satisfying the planned outcome.

### No severe regression
Block obvious broken logic, missing critical error handling, or unintended scope creep — whether in new code or removed from existing code.

Do not flag: minor style differences, harmless refactors, or plan drift when objectives remain met.

### Implementation correctness
New code must be fundamentally correct: no obviously broken logic, missing critical error handling, or incorrect behavior visible in the diff.

Bad: new async path ignores rejected promise.
Good: async path handles expected failures or propagates them safely.
