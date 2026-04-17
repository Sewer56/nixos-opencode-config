---
description: "Plan a parameterised test refactor for target file(s)"
agent: plan
---

# Parameterise Tests

Create a confirmation-first plan to convert repetitive tests into parameterised
tests in the target file(s). This command plans only and does not edit files.

## User Input

```text
$ARGUMENTS
```

Use `$ARGUMENTS` as one or more file or directory paths.

If no target path is provided, stop and ask for an explicit path.

## Process

`TESTING_RULES_PATH`: `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/config/rules/testing.md`
`TEST_PARAMETERIZATION_RULES_PATH`: `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/config/rules/test-parameterization.md`

1. Read `TESTING_RULES_PATH` and `TEST_PARAMETERIZATION_RULES_PATH` once and use them as the source of truth for duplicate-coverage expectations, case naming, and labels/comments.

2. Resolve targets
- If a file is provided, use it directly.
- If a directory is provided, find test files in scope based on project
  conventions.

3. Read and group tests
- Read each target file fully.
- Group tests that exercise the same logic path but vary by data only.

4. Choose strategy per group
- Use the existing test framework for each file.
- Prefer native parameterisation support in that framework.
- Apply the parameterize-vs-split rules from `TEST_PARAMETERIZATION_RULES_PATH`.

5. Draft the plan (no edits)
- For each file, list each candidate group and proposed replacement strategy (parameterized or split).
- For parameterized tests, include:
  - test function name
  - case names
  - planned parameters in order
- For split proposals, include:
  - shared helper name and signature
  - individual test names

6. Verification plan
- Include exact commands to run after implementation.
- Prefer repo verification scripts when available.

7. Confirmation gate
- Return the plan and stop.
- Ask for confirmation: `Say "go" to apply this plan, or suggest changes.`
- Show proposed changes for every target file (not just selected files).

## Output Format (Template)

````markdown
Proposed Parameterisation Plan

Targets:
- <path>

## Parameterisation Summary
| Metric | Value |
|---|---:|
| Files in scope | <n> |
| Test functions (before) | <n> |
| Test functions (after) | <n> |
| Net change | <+/-n> |
| Parameterised test functions | <n> |
| Named cases added | <n> |
| Coverage intent | Equal or better |

## File Delta
| File | Before | After | Main change |
|---|---:|---:|---|
| <path> | <n> | <n> | <summary> |

## Changes (Diff-Ready, All Files)

Include one `### <path>` block for every file in `Targets`. Do not skip files.
If a file should remain unchanged, say so explicitly and explain why.

### <path>
Group:
- <what is repetitive>

Before:
```<language>
<old tests being replaced>
```

After (planned):
```<language>
<new parameterised test with case names, parameters, and comments>
```

Parameter labelling (only when non-obvious):
- Add labels directly beside parameters in the `After (planned)` code block.

Verification:
- <command>

Say "go" to apply this plan, or suggest changes.
````

## Constraints
- Keep the plan implementation-ready and concrete.
- Keep behaviour and coverage at least equivalent.

## Example Output

````markdown
Proposed Parameterisation Plan

Targets:
- src/math.rs

## Parameterisation Summary
| Metric | Value |
|---|---:|
| Files in scope | 1 |
| Test functions (before) | 3 |
| Test functions (after) | 1 |
| Net change | -2 |
| Parameterised test functions | 1 |
| Named cases added | 3 |
| Coverage intent | Equal or better |

## File Delta
| File | Before | After | Main change |
|---|---:|---:|---|
| math.rs | 3 | 1 | grouped sign-check tests by data variation |

## Changes (Diff-Ready, All Files)

### src/math.rs
Group:
- Sign-check tests with identical logic, differing only in input/expected.

Before:
```rust
#[test]
fn absolute_value_should_flip_negative() { ... }

#[test]
fn absolute_value_should_keep_zero() { ... }

#[test]
fn absolute_value_should_keep_positive() { ... }
```

After (planned):
```rust
#[rstest]
#[case::negative_should_flip(
    -5,  // input: negative value
    5    // expected: sign flipped
)]
#[case::zero_should_be_unchanged(
    0, // input: zero
    0  // expected: no change
)]
#[case::positive_should_be_unchanged(
    7, // input: positive value
    7  // expected: no change
)]
fn absolute_value_should_return_expected(
    #[case] input: i32,
    #[case] expected: i32,
) { ... }
```

Verification:
- cargo test

Say "go" to apply this plan, or suggest changes.
````
