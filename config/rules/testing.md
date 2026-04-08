## Testing Rules

Use these rules when the task requires tests.

- Cover all new code with tests.
- Avoid duplicate coverage and setup; do not restate what an existing test already proves.
- Reuse existing test helpers; extract shared helpers only when they reduce repetition or clarify setup across multiple tests.
- Keep tests deterministic; avoid real I/O, time, and network unless controlled, seeded, or frozen.
- When one behavior needs multiple similar cases, follow `test-parameterization.md` for naming, labels, and case structure.
- Name tests as behavioral claims: `subject_should_expectation_when_condition`. Use the language's standard identifier style.
- Omit the `when` clause for simple cases; include it for conditional or edge-case behavior.
- Drop redundant prefixes (`test_`, `rule_`) when the module already provides context.
- Group related tests with lightweight section comments (e.g. `// --- section ---`).
- Order: construction → core behavior → edge cases → convenience.

### Example

```rust
#[test]
fn glob_should_return_matching_files_when_pattern_matches() { ... }
fn glob_should_return_empty_when_pattern_matches_nothing() { ... }

#[rstest]
#[case::matches_extension("*.rs", "lib.rs", true)]
#[case::excludes_gitignored("build/", "output.rs", false)]
fn glob_pattern_includes_or_excludes_files(#[case] p: &str, #[case] n: &str, #[case] ok: bool) { ... }
```
