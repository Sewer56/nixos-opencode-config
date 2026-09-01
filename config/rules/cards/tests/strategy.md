### Acceptance lens
Tests must target behavior the implementation could get wrong, not compiler-guaranteed facts.

Assert observable behavior tied to acceptance criteria.

Tests name acceptance behavior, never labels or internal ids (`AC-1`).

### Changed behavior coverage
Cover critical new or changed behavior: success, failure, edge cases. Cover all new code when the task requires tests.

### Differential tests for equivalence claims
Equivalence claims need one test running both paths and asserting equal final rendered/consumed results; request-shape mocks prove nothing about equivalence.

### Redundancy
- **No duplicates**: Never restate what an existing test proves. Before adding
  a test, fold its assertions into one with the same setup and entry point,
  or parameterize that test.
- **Intentional repetition**: Coverage repeated across public entry points is
  intentional; do not flag it.
- **Map surviving homes**: Removing a redundant test requires mapping every
  assertion to a surviving home.
- **Examples are not coverage**: Example binaries never substitute for tests.

### Test helpers
Reuse existing helpers; extract shared ones only to reduce repetition or
clarify setup across tests.

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
