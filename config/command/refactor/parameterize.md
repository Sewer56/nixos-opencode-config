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

## Shared Rules

- `TESTING_RULES_PATH`: `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/config/rules/testing.md`
- `TEST_PARAMETERIZATION_RULES_PATH`: `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/config/rules/test-parameterization.md`

## Process

1. Read `TESTING_RULES_PATH` and `TEST_PARAMETERIZATION_RULES_PATH` once and use them as the source of truth for duplicate-coverage expectations, case naming, and labels/comments.

2. Resolve targets
- If a file is provided, use it directly.
- If a directory is provided, find test files in scope based on project
  conventions.

3. Read and group tests
- Read each target file fully.
- Group tests that exercise the same logic path but vary by data only.

4. Choose framework-specific strategy
- Use the existing test framework for each file.
- Prefer native parameterisation support in that framework.

5. Draft the plan (no edits)
- For each file, list each candidate group and proposed replacement test.
- For each proposed parameterised test, include:
  - test function name
  - case names
  - planned parameters in order
  - parameter labels/comments style

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
- Add labels directly beside `#[case]` parameters in the `After (planned)` code
  block (for example: `#[case] include: Option<&str>, // optional glob filter`).

Verification:
- <command>

Say "go" to apply this plan, or suggest changes.
````

## Example Output (From This Repo Style)

````markdown
Proposed Parameterisation Plan

Targets:
- llm-coding-tools-core/src/tools/grep.rs
- llm-coding-tools-core/src/path/allowed.rs

## Parameterisation Summary
| Metric | Value |
|---|---:|
| Files in scope | 2 |
| Test functions (before) | 14 |
| Test functions (after) | 9 |
| Net change | -5 |
| Parameterised test functions | 4 |
| Named cases added | 13 |
| Coverage intent | Equal or better |

## File Delta
| File | Before | After | Main change |
|---|---:|---:|---|
| tools/grep.rs | 6 | 5 | grouped repetitive search and edge-case checks |
| path/allowed.rs | 8 | 4 | grouped valid resolution and traversal rejection checks |

## Changes (Diff-Ready, All Files)

### llm-coding-tools-core/src/tools/grep.rs
Group:
- Basic search behaviour tests with the same setup but different data.

Before:
```rust
#[test]
fn grep_finds_matches() { ... }

#[test]
fn grep_respects_glob_filter() { ... }
```

After (planned):
```rust
#[rstest]
#[case::single_file_no_filter(
    vec![("match.txt", "hello world")], // files: 1 file with 1 match
    "hello",                            // pattern: matches 1 place
    None::<&str>,                       // filter: none (search all)
    1,                                  // expected: 1 file matched
    1                                   // expected: 1 total match
)]
#[case::glob_filters_to_rs_only(
    vec![("match.rs", "hello"), ("match.txt", "hello")], // files: 2 files
    "hello",                                             // pattern: matches 1+1 places
    Some("*.rs"),                                        // filter: only .rs files
    1,                                                   // expected: 1 file matched
    1                                                    // expected: 1 total match
)]
fn grep_search_finds_expected_matches(
    #[case] files: Vec<(&str, &str)>,
    #[case] pattern: &str,
    #[case] include: Option<&str>, // optional glob filter
    #[case] expected_file_count: usize, // files with at least one match
    #[case] expected_match_count: usize, // total matches across files
) { ... }
```

### llm-coding-tools-core/src/path/allowed.rs
Group:
- Path resolution tests with identical resolver setup and assertion style.

Before:
```rust
#[test] fn resolves_relative_path_in_allowed_dir() { ... }
#[test] fn resolves_nested_path() { ... }
#[test] fn allows_non_existent_path_for_write() { ... }
```

After (planned):
```rust
#[rstest]
#[case::existing_file_in_root("file.txt", "file.txt")]  // exists: setup_test_dir()
#[case::nested_existing_file("subdir/nested.txt", "nested.txt")]  // exists: setup
#[case::new_file_in_root("new_file.txt", "new_file.txt")]  // does NOT exist
fn resolves_valid_paths_successfully(
    #[case] input_path: &str,
    #[case] expected_filename: &str, // expected suffix after resolution
) { ... }
```

Verification:
- bash /home/sewer/Project/llm-coding-tools/src/.cargo/verify.sh

Say "go" to apply this plan, or suggest changes.
````

## Constraints
- Keep the plan implementation-ready and concrete.
- Keep behaviour and coverage at least equivalent.
