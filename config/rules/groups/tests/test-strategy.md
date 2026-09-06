## Test Strategy

Test implementation behavior, not compiler guarantees.
Assert observable behavior tied to acceptance criteria.

Cover critical changes: success, failure, and edge cases.
Cover all new code when the task requires tests.

Equivalence claims need one test executing both paths and asserting equal final rendered/consumed results.
Request-shape mocks do not prove equivalence.

Do not add redundant coverage except across public entry points.
Repeated coverage there is intentional and must not be flagged.

Before adding tests, reuse tests matching setup and entry point.
Fold in assertions.
Parameterize when all cases make one claim and only data varies.
Use separate tests with a shared helper if claims differ or no single name fits.

Name each case descriptively.
Keep argument order stable: primary input → mode/flags → expected output.
Comment only non-obvious parameters or assertions.
Keep cases human-friendly around 80-100 characters per line.

Map every removed redundant assertion to a surviving test.
Examples never replace tests.

Reuse helpers.
Extract shared helpers only to reduce repetition or clarify setup across tests.
Prefer one parameterizable local helper over per-test mock structs.

Keep tests deterministic.
Avoid real I/O, time, and network unless controlled, seeded, or frozen.

Name tests by acceptance behavior, not labels or internal IDs.
Use `subject_should_expectation_when_condition` in language identifier style.

Include `when` for conditional or edge behavior, otherwise omit it.
Drop module-redundant prefixes.

Group related tests with lightweight section comments.
Order tests: construction → core behavior → edge cases → convenience.
