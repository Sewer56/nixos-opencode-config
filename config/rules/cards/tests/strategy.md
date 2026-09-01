### Acceptance lens
Tests must target behavior the implementation could get wrong, not compiler-guaranteed facts.

Assert observable behavior tied to acceptance criteria.

Tests name acceptance behavior, never labels or internal ids (`AC-1`).

### Changed behavior coverage
Cover critical new or changed behavior, including success, failure, and relevant edge cases. Cover all new code when the task requires tests.

### Differential tests for equivalence claims
For a claim that two paths produce equivalent output, one test executes both paths and asserts equality of the final rendered or consumed result; mocks capturing request shape do not establish rendered equivalence.

### Redundancy
- **No duplicate coverage**: Avoid duplicate coverage and setup; do not
  restate what an existing test already proves.
- **Append, do not fork**: When new assertions share an existing test's setup
  and entry point, append them there rather than creating a separate function.
- **Intentional repetition**: Do not flag intentionally repeated coverage
  across different public entry points.
- **Check fold-in first**: Before adding a test, check whether its unique
  assertions fold into an existing test with the same setup and entry point,
  or whether an existing test can be parameterized to cover them.
- **Map surviving homes**: When removing a redundant test, map every assertion
  to a surviving home.
- **Examples are not coverage**: Example binaries never substitute for test
  coverage.

### Test helpers
Reuse existing test helpers. Extract shared helpers only when they reduce repetition or clarify setup across multiple tests.

Prefer one parameterizable local helper over re-declared per-test mock
structs: closure adapters, failure constructors (`fail_hook(msg)`), test
builders, shared stub modules.

### Determinism
Keep tests deterministic. Avoid real I/O, time, and network unless controlled, seeded, or frozen.

### Behavioral names
Name tests as behavioral claims: `subject_should_expectation_when_condition`, using the language's standard identifier style.

Omit `when` for simple cases; include it for conditional or edge-case behavior.

Drop redundant prefixes when the module already provides context.

### Organization
Group related tests with lightweight section comments. Order tests: construction → core behavior → edge cases → convenience.
