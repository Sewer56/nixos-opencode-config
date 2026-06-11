### Implementation fidelity
Changes must satisfy the described outcome, files, and anchors. Exact text is not required when behavior is equivalent.
Block unrelated edits that omit planned anchor behavior.

### No severe regression
Block obvious broken logic, missing critical error handling, and unintended scope creep introduced or exposed by the change.
Allow minor style differences, harmless refactors, and behavior-equivalent plan drift.

### Functional correctness
Run the program's build and test suite when available. Capture command, exit code, and key output.
Block failing builds/tests or observed runtime errors; pass when checks succeed and expected behavior is observable.
