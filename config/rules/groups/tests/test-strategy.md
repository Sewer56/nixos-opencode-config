## Test Strategy

### Acceptance and coverage
Test behavior the implementation could get wrong, not compiler-guaranteed facts.
Assert observable behavior tied to acceptance criteria.

Cover critical new/changed behavior: success, failure, and edge cases.
Cover all new code when the task requires tests.

Equivalence claims need one test executing both paths and asserting equal final rendered/consumed results.
Request-shape mocks do not prove equivalence.

### Redundancy
- Never restate what an existing test proves.
- Before adding a test, fold assertions into one with the same setup and entry point, or parameterize it.
- Coverage repeated across public entry points is intentional; do not flag it.
- Removing a redundant test requires mapping every assertion to a surviving home.
- Example binaries never substitute for tests.

### Test helpers and determinism
Reuse existing helpers; extract shared ones only to reduce repetition or clarify setup across tests.

Prefer one parameterizable local helper over re-declared per-test mock structs.
Options include closure adapters, failure constructors (`fail_hook(msg)`), test builders, and shared stub modules.

Keep tests deterministic; avoid real I/O, time, and network unless controlled, seeded, or frozen.

### Names and organization
Name tests as acceptance behavior, never labels or internal ids (`AC-1`).
Use `subject_should_expectation_when_condition` in the language's standard identifier style.

Omit `when` for simple cases; include it for conditional or edge-case behavior.
Drop redundant prefixes when the module provides context.

Group related tests with lightweight section comments.
Order tests: construction → core behavior → edge cases → convenience.
