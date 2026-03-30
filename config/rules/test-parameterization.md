# Test Parameterization Rules

Use these rules when a single behavior needs multiple similar test cases.

- Prefer parameterized tests when multiple inputs exercise the same logic path; keep separate tests only when setup, assertions, or failure modes differ.
- When planning parameterized tests, include representative case naming and parameter labeling style (for example `empty_input_returns_zero`).
- Give each case a descriptive name; avoid generic names like `case_1`.
- Keep argument order stable: primary input -> mode/flags -> expected output.
- Label parameters with short plain-English comments only when the meaning is non-obvious.
- Keep labels aligned where practical.
- If inline labels become too long, move labels above the case.
- Add occasional in-body comments for non-obvious setup or assertions.
- Keep tests human-friendly and around 80-100 characters per line.
- For Rust, prefer `rstest` with `#[case::name(...)]` and aligned labeled parameters/comments.

## Rust Style Reference

```rust
/// Verifies that line truncation in formatted output behaves correctly for
/// different line lengths and line number settings.
#[rstest]
#[case::with_line_numbers_short(
    6,           // max_len: line "abcdefghij" (10 chars) truncated to 6
    true,        // with_line_numbers: yes, shows "L1: " prefix
    "L1: abc..." // expected: truncated with line number prefix
)]
#[case::without_line_numbers_short(
    4,        // max_len: line truncated to 4 chars
    false,    // with_line_numbers: no prefix
    "  a..."  // expected: truncated without line number prefix
)]
#[case::no_truncation_when_fits(
    200,             // max_len: larger than line length (10 chars)
    true,            // with_line_numbers: yes
    "L1: abcdefghij" // expected: full line preserved, no truncation
)]
#[case::exact_boundary_no_truncation(
    10,              // max_len: exactly matches line length (10 chars)
    true,            // with_line_numbers: yes
    "L1: abcdefghij" // expected: full line preserved, boundary not exceeded
)]
fn grep_format_handles_line_truncation(
    #[case] max_len: usize,
    #[case] with_line_numbers: bool,
    #[case] expected: &str,
) {
    // Keep setup short; comment only non-obvious assertions.
}
```
