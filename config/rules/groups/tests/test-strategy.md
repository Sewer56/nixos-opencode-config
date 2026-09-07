## Test Strategy

Test observable acceptance behavior, not compiler guarantees.

Cover critical success, failure, and edge cases.
Cover all new code when tests are required.

Equivalence claims need one test executing both paths and asserting equal final rendered/consumed results.
Request-shape mocks do not prove equivalence.

Allow redundancy only across public entry points; never flag it there.

First extend tests matching setup and entry point.

Parameterize independent cases making one claim with data-only variation.
Use named framework cases, not data loops.

Use a parameterization framework, such as Rust's rstest; add it if needed.
Keep separate tests if claims differ or no single name fits.

Allow loops intrinsic to one stateful scenario or assertion.

Name cases descriptively.
Order arguments: primary input → mode/flags → expected output.
Comment only non-obvious parameters or assertions.
Keep readable cases around 80-100 columns.

Map removed redundant assertions to surviving tests.
Examples never replace tests.

Reuse helpers; extract only for repetition or shared setup clarity.
Prefer one parameterizable local helper over per-test mock structs.

Ensure determinism; control, seed, or freeze real I/O, time, and network.

Name tests by acceptance behavior, not labels or IDs.
Use `subject_should_expectation_when_condition` in language identifier style.

Use `when` only for conditional or edge behavior.
Drop module-redundant prefixes.

Group related tests with lightweight section comments.
Order tests: construction → core behavior → edge cases → convenience.
