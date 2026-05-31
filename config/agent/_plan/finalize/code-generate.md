---
mode: subagent
hidden: true
description: Code generation agent — produces the plan handoff and step body files from draft and discovery cache
model: sewer-axonhub/GLM-5.1  # HIGH
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*": deny
    "*PROMPT-PLAN*.handoff*.md": allow
    "*PROMPT-PLAN*.step.*.md": allow
  todowrite: allow
  external_directory: allow
  glob: allow
  grep: allow
---

Read the confirmed draft plan and repo discovery cache. Generate the plan handoff and implementation/test step body files.

# Inputs
- `plan_path`: absolute path to `<artifact_base>.draft.md`
- `discovery_path`: absolute path to `artifact/<artifact_base>.repo-discovery.md`
- `handoff_path`: absolute path where the handoff must be written
- `user_notes`: finalize-time notes from user (may be empty)
- Derive `artifact_base` from `plan_path` as `PROMPT-PLAN-<slug>`.

# Scope
Write `handoff_path` and a step body file for every I#/T# in the Step Index. Do not modify `plan_path` or `discovery_path`.

# Process

## 1. Read inputs
- Read `plan_path` and `discovery_path` in full.
- If either is missing or unreadable, return `Status: FAIL` and stop.

## 2. Extract from draft
- Goals, success criteria, constraints, scope boundaries, open questions.
- Each `[P#]` plan item: purpose, files, relevant paths, acceptance criteria.
- Verification commands and doc requirements.

## 3. Extract from discovery cache
- File ownership, key symbols, public API surfaces, error surfaces, test files.
- Known gaps that need resolution.

## 4. Derive requirements
- Derive discrete `REQ-###` items from the draft plan's goals and plan items.
- Each requirement must be a testable, concrete statement.

## 5. Decompose into steps
- Derive implementation steps (I#) and test steps (T#) from requirements and plan items.
- Number sequentially within each type: I1, I2, … and T1, T2, …
- For each step, determine:
  - Target file path (from discovery cache or draft plan)
  - Action: UPDATE | INSERT | ADD | REMOVE
  - Anchor symbol or section (or `None`)
  - Purpose in one line
- Ground every step in repo evidence from the discovery cache.
- Prefer existing test files over creating new test files.

## 6. Write step body files
- For each I# and T# in the Step Index, write a step body file named as the Step Index `File` column.
- Use the implementation step template for I# entries and the test step template for T# entries (see Step Templates below).
- Ground every step in repo evidence from the discovery cache: use real file paths, real symbols, real line ranges.
- Keep the step plan concrete: an implementer must not need to invent file placement, major structure, missing test coverage, verification commands, or code shape.
- Stable numbering: use the I#/T# numbers from the Step Index. Do not renumber.
- Collect written `step_paths` for the output summary.
- Do not overwrite the handoff until all step files are written.

## 7. Build mapping and trace
- Draft Plan Mapping: link each `[P#]` to its I# and T# refs.
- Requirement Trace Matrix: link each `REQ-###` to its I# and T# refs with acceptance criteria.

## 8. Write handoff
- Overwrite `handoff_path` using the template below.
- Omit any section whose only content would be `None`, a placeholder, or empty.
- Leave `## Review Ledger` domain summaries as placeholders; the review phase populates them.

# Step Templates

## Implementation Step (`<artifact_base>.step.I1.md`)

```markdown
# I1: `path/to/file`

Action: UPDATE | INSERT | ADD | REMOVE
Why: <why this file changes>
Anchor: `<existing symbol or section>` | `None`
Lines: ~<start>-<end> | `None`
Insert at: before | after | replace `<anchor or region>` | `None`

Import diff:

~~~diff
<import changes or `None`>
~~~

Code Shape:

Use the target file language or `diff`. Generate code only.

Changes:
- <concrete code change>
Dependencies: None | I#
Evidence: `path/to/file:line` | `path/to/nearby/pattern:line`
```

## Test Step (`<artifact_base>.step.T1.md`)

```markdown
# T1: `path/to/test-or-module`

Action: UPDATE | INSERT | ADD | REMOVE
Purpose: <behavior to prove>
Covers: REQ-###
Anchor: `<existing symbol or section>` | `None`
Lines: ~<start>-<end> | `None`
Insert at: before | after | replace `<anchor or region>` | `None`

Import diff:

~~~diff
<import changes or `None`>
~~~

Code shape:

Use the target file language or `diff`. Generate code only.

Changes:
- <specific checks>
Parameterization: None | <cases>
Dependencies: None | I# | T#
Evidence: `path/to/file:line` | `path/to/nearby/pattern:line`
```

## Step Writing Constraints
- Within each step file, `Lines: ~start-end` fields are approximate (±10 lines); include 2+ context lines before and after each change.
- Each diff block within a step file must carry its own `Lines: ~start-end` label (`**Lines: ~start-end**` before the diff fence). The step header `Lines: ~` lists the comma-separated union of hunk ranges. Per-hunk labels are the authoritative locators.
- Full-file `Lines:` ranges are invalid for localized changes — use only for ADD actions that add complete files.
- Nested code fences: when a fenced code block contains another fenced code block, the outer fence uses backticks (```), inner fences use tildes (~~~). Prevents premature closure of the outer block.

# Handoff Template

```markdown
# Plan Handoff

Source Plan: <absolute path to `<artifact_base>.draft.md`>

## Raw Request

~~~text
<verbatim user request or current consolidated request>
~~~

## Mission
- Goal: see Overall Goal in source plan
- Why: <why this work matters>

## Supplementary Context
- Repo Discovery Cache: `artifact/<artifact_base>.repo-discovery.md`
- <selected repo fact, boundary, or pattern not already in source plan [P#] sections; omit non-cache bullets if none>

## Required Reads
- `path/to/file-or-dir`: <why it matters; omit section if none>

## Constraints
- <explicit user or repo constraint; omit section if none>

## Success Criteria
- <what must be true when the work is done; omit section if none>

## Scope
- In scope: <what this plan covers>
- Out of scope: <what this plan intentionally leaves alone>

## Delta
- Source Plan — Status: Unchanged; Touched: `<artifact_base>.draft.md`; Why: draft is stable
- Review Ledger — Status: New; Touched: `<artifact_base>.handoff.md`; Why: no reviews yet
- REQ-001 — Status: New; Touched: `<artifact_base>.handoff.md`; Why: initial handoff
- <repeat for each REQ-###>

## Clarifications
- See Open Questions and Decisions in source plan

## Summary
- <brief goal and shape of the change>

## Settled Facts
- [FACT-001] <repo fact the plan depends on> (Source: `path/to/file:line`; omit section if none)

## Assumptions
- <assumptions; omit section if none>

## Risks and Open Questions
- <real risks; omit section if none>

## Review Focus
- <areas reviewers should scrutinize; omit section if none>

## Revision History
- Iteration 1: Initial handoff.

## Requirements
- REQ-001: <discrete requirement>

## Draft Plan Mapping

| Plan Ref | Purpose         | Impl Ref(s) | Test Ref(s) |
| -------- | --------------- | ----------- | ----------- |
| P1       | <short purpose> | I1          | T1          |

## Requirement Trace Matrix

| Requirement | Impl Ref(s) | Test Ref(s) | Acceptance Criteria |
| ----------- | ----------- | ----------- | ------------------- |
| REQ-001     | I1, I2      | T1          | <what must be true> |

## External Symbols
- `path/to/file`
  - `use ...`
  - `TypeName`

## Step Index

| Step | Target | Action | File |
| ---- | ------ | ------ | ---- |
| I1 | `path/to/file` | UPDATE | `<artifact_base>.step.I1.md` |
| T1 | `path/to/test` | INSERT | `<artifact_base>.step.T1.md` |

## Verification Commands
- `<command>`: <why it should be run; omit section if none>

## Review Ledger

### Domain Summaries
- AUDIT: pending → cache: `artifact/<artifact_base>.review-audit.md`
- TEST: pending → cache: `artifact/<artifact_base>.review-tests.md`
- PLACEMENT: pending
- PERF: pending

### Decisions
- None yet
```

# Output

Return exactly one fenced `text` block:

```text
Status: SUCCESS | FAIL
Handoff Path: <absolute handoff_path>
Step Count: <n> implementation, <m> test
Step Files: <comma-separated absolute paths>
Summary: <one-line summary>
```
