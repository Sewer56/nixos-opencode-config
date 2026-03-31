# Test Parameterization Rules

Use these rules when a single behavior needs multiple similar test cases.

- Prefer parameterized tests for multiple inputs on the same logic path; use separate tests only when setup, assertions, or failure modes differ.
- Give each case a descriptive name and parameter labeling style (e.g. `empty_input_returns_zero`); avoid generic names like `case_1`.
- Keep argument order stable: primary input -> mode/flags -> expected output.
- Label parameters with short comments only when non-obvious.
- Keep labels aligned where practical.
- If inline labels become too long, move labels above the case.
- Comment non-obvious setup or assertions inline.
- Keep tests human-friendly and around 80-100 characters per line.
- For Rust: prefer `rstest` with `#[case::name(...)]` and aligned parameters/comments.

## Style Reference

```rust
/// Verifies line truncation in formatted output.
#[rstest]
#[case::with_line_numbers(
    6,           // max_len: truncate "abcdefghij" (10 chars) to 6
    true,        // with_line_numbers: yes, shows "L1: " prefix
    "L1: abc..." // expected: truncated with line number prefix
)]
#[case::without_line_numbers(
    4,        // max_len: truncate to 4 chars
    false,    // with_line_numbers: no prefix
    "  a..."  // expected: truncated without prefix
)]
fn grep_format_handles_line_truncation(
    #[case] max_len: usize,
    #[case] with_line_numbers: bool,
    #[case] expected: &str,
) {
    // Keep setup short; comment only non-obvious assertions.
}
```
