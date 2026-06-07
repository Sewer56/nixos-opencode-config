### Acceptance lens
Tests must prove behavior the implementation could get wrong. Never write tests that only assert what the compiler or type system already guarantees (e.g., field access after assignment, enum variant identity, trivial delegation).
Bad: field-access test (`assert_eq!(ctx.field, "val")`), enum-variant-existence test (`assert_eq!(Variant::A, Variant::A)`), asserting private helper call order.
Good: test asserts observable behavior tied to an acceptance criterion.

### Changed behavior coverage
Cover critical new or changed behavior, including success, failure, and relevant edge cases. Cover all new code when the task requires tests.
Bad: new error path has no test.
Good: test covers success, failure, and relevant edge case.

### Redundancy
Avoid duplicate coverage and setup; do not restate what an existing test already proves.
Do not flag: intentionally repeated coverage across different public entry points.

### Test helpers
Reuse existing test helpers. Extract shared helpers only when they reduce repetition or clarify setup across multiple tests.

### Determinism
Keep tests deterministic. Avoid real I/O, time, and network unless controlled, seeded, or frozen.

### Behavioral names
Name tests as behavioral claims: `subject_should_expectation_when_condition`, using the language's standard identifier style. Omit `when` for simple cases; include it for conditional or edge-case behavior. Drop redundant prefixes when the module already provides context.

### Organization
Group related tests with lightweight section comments. Order tests: construction → core behavior → edge cases → convenience.
