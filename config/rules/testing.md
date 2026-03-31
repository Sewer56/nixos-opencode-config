# Testing Rules

Use these rules when the task requires tests.

- Cover all new code with tests.
- Avoid duplicate coverage and setup; do not restate what an existing test already proves.
- Reuse existing test helpers; extract shared helpers only when they reduce repetition or clarify setup across multiple tests.
- Keep tests deterministic; avoid real I/O, time, and network unless controlled, seeded, or frozen.
- When one behavior needs multiple similar cases, follow `test-parameterization.md` for naming, labels, and case structure.
