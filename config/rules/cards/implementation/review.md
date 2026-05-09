### Implementation fidelity
Changes should match the described code shape and anchors without requiring exact textual adherence when behavior is equivalent.
Bad: implementation edits unrelated files and omits planned anchor behavior.
Good: implementation uses equivalent helper placement while satisfying the planned outcome.

### No severe regression
Block obvious broken logic, missing critical error handling, or unintended scope creep, whether new or removed from existing code.
Do not flag: minor style differences, harmless refactors, or plan drift when the implementation's behavior is equivalent.

### Functional correctness
Run the program's build and test suite. Capture output and exit codes.
Bad: build fails, tests fail, or runtime error observed on execution.
Good: build succeeds, tests pass, and expected behavior is observable from command output.
