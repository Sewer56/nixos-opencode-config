## Test Parameterization Rules

Use these rules when a single behavior needs multiple similar test cases.

- Parameterize when all cases share the same behavioral claim and only data varies.
- Split into individual tests with a shared helper when each case makes a distinct behavioral claim.
- If the function name cannot equally describe all cases, split instead.
- Give each case a descriptive name (e.g. `star_should_match_empty`); avoid generic names like `case_1`.
- Keep argument order stable: primary input → mode/flags → expected output.
- Label parameters with short comments only when non-obvious; use simple `#[case::label("value1", "value2", true)]` style for obvious cases.
- Comment non-obvious setup or assertions inline.
- Use simple form when parameters are obvious:
  ```
  #[case::positive("abc", true)]
  #[case::negative("xyz", false)]
  ```
- Use expanded form with aligned comments when behavior needs explanation:
  ```
  #[case::label(
      "value1", // param_a: why this value
      "value2", // param_b: why this value
      true      // expected: why this result
  )]
  ```
- Move labels above the case if inline becomes too long.
- Keep tests human-friendly, around 80–100 characters per line.
