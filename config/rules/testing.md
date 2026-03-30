# Testing Rules

Use these rules when the task requires tests.

- Add or update sufficient tests so all new code introduced by the change is covered.
- Avoid duplicate coverage and duplicate setup; if an existing test already proves a behavior, do not restate it elsewhere.
- Reuse existing test helpers when they fit; extract a shared helper only when it reduces repetition or makes setup clearer across multiple tests.
- Keep tests deterministic; avoid real I/O, time, and network unless the test deliberately controls, seeds, or freezes them.
- When one behavior needs multiple similar cases, prefer parameterized tests and follow `test-parameterization.md` for naming, labels, and case structure.
