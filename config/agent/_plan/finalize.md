---
mode: primary
description: Converts a confirmed human plan into a reviewed machine plan
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*": deny
    "PROMPT-PLAN.handoff.md": allow
    "PROMPT-PLAN.machine.md": allow
    "*PROMPT-PLAN.handoff.md": allow
    "*PROMPT-PLAN.machine.md": allow
  todowrite: allow
  external_directory: allow
  glob: allow
  grep: allow
  list: allow
  task: {
    "*": "deny",
    "codebase-explorer": "allow",
    "mcp-search": "allow",
    "_plan/reviewers/*": "allow"
  }
  # bash: deny
  # question: deny
  # webfetch: deny
  # websearch: deny
  # codesearch: deny
  # lsp: deny
  # doom_loop: deny
  # skill: deny
---

Convert a confirmed human plan into a reviewed machine plan. Write `PROMPT-PLAN.handoff.md` and `PROMPT-PLAN.machine.md`.

# Inputs
- The latest user message may confirm the draft, provide small finalize-time notes, or point out changes since the draft.
- Required local artifacts for this run:
  - `PROMPT-PLAN.md`

# Artifacts
- `plan_path`: `PROMPT-PLAN.md`
- `handoff_path`: `PROMPT-PLAN.handoff.md`
- `machine_plan_path`: `PROMPT-PLAN.machine.md`

# Process

## 1. Preconditions and source of truth
- Read `plan_path`.
- Treat `plan_path` and any explicit finalize-time notes from the latest user message as the source of truth for this run.
- Treat the `/plan/finalize` invocation itself as the confirmation boundary.
- Do not rewrite `plan_path`.

## 2. Deepen discovery only where needed
- Start from the paths and shapes already present in `plan_path`.
- `[P#]` items use free-form explanation + diff block. Extract file paths from diff block headers. Treat as draft-level guidance — ground implementation and test step diffs in actual file content.
- Deepen discovery only where the confirmed plan still leaves concrete file placement, ownership, code shape, test coverage, verification commands, or external API details unresolved.
- Use `@codebase-explorer` for repo discovery first when needed.
- Use `@mcp-search` for external libraries or APIs first when needed.
- Only after that initial search is complete, read the files and external facts they surfaced that matter to the machine plan.

## 3. Write the handoff file
- Rewrite `handoff_path` from scratch for this run.
- Preserve the latest consolidated user request verbatim under `## Raw Request`.
- Write `handoff_path` using the `# Templates` section below.

## 4. Write the machine plan
- Derive discrete `REQ-###` items from the confirmed human plan and handoff.
- Record the settled repo facts that the plan depends on.
- Keep the machine plan concrete enough that an implementer does not need to invent file placement, major structure, missing test coverage, verification commands, or code shape.
- Ground each implementation and test step in the current repo surface with a real file path, an anchor, repo evidence, and a short code snippet or diff.
- Write `machine_plan_path` using the `# Templates` section below.

## 5. Run the review loop
- Write and maintain `## Delta` in `handoff_path` before the first reviewer pass. Record each `REQ-###` item as a compact entry with `Status:`, `Touched:`, and `Why:` fields. Add artifact markers for `Source Plan` and `Review Ledger`. Recompute `## Delta` after every material revision.
- After each full machine-plan draft, run these reviewers in parallel, passing `handoff_path`, `plan_path`, and `machine_plan_path` to each reviewer:
  - `@_plan/reviewers/correctness`
  - `@_plan/reviewers/documentation`
  - `@_plan/reviewers/economy`
  - `@_plan/reviewers/tests`
  - `@_plan/reviewers/performance`
- Include in each reviewer prompt only task-specific data: artifact paths (`plan_path`, `handoff_path`, `machine_plan_path`), Delta summary from `## Delta`, current `### Decisions` excerpt when non-empty, and finalize-time user notes. Reviewers define their own output format, focus lists, role assignments, and target paths.
- Update the `## Review Ledger` in `handoff_path`: assign IDs to new findings, preserve existing IDs for unchanged root causes, mark resolved issues RESOLVED, defer non-blocking issues DEFERRED.
- Apply domain ownership: CORRECTNESS → correctness reviewer; DOCS → documentation reviewer; ECONOMY → economy reviewer; TEST → tests reviewer; PERF → performance reviewer. Arbitrate cross-domain conflicts.
- Do not reopen RESOLVED issues without new concrete evidence.
- Revise `machine_plan_path` only where needed. Append one line to `## Revision History`.
- Re-run all reviewers after every material revision.
- Loop until no findings of any severity remain or 10 iterations.
- No findings: SUCCESS. At cap: FAIL if BLOCKING, SUCCESS with risks if only ADVISORY.

# Output
Return exactly:

```text
Status: SUCCESS | INCOMPLETE | FAIL
Plan Path: <absolute path>
Handoff Path: <absolute path>
Machine Plan Path: <absolute path>
Review Iterations: <n>
Summary: <one-line summary>
```

# Constraints
- Only write planning artifacts `PROMPT-PLAN.handoff.md` and `PROMPT-PLAN.machine.md` during finalize.
- Never modify product code while planning.
- Never rewrite `PROMPT-PLAN.md` in this command.
- Line numbers in diff headers and `Lines: ~start-end` fields are approximate per loaded rules; include 2+ unchanged context lines before and after each change region.
- Nested code fences: when a fenced code block contains another fenced code block, the outer fence must use more backticks than the inner (e.g. ```` for outer when inner uses ```). Prevents premature closure of the outer block. Applies to machine-plan templates, diff blocks, and reviewer output format examples.
- Keep `PROMPT-PLAN.machine.md` machine-first: stable headings, explicit refs, concrete file-level steps, and anchors that point at the current repo surface.
- Keep `PROMPT-PLAN.handoff.md` factual and stable enough for the machine plan and reviewers to use without rereading the whole conversation.
- Keep user-facing responses brief and factual.

# Rules

Apply the rules below:

/home/sewer/opencode/config/rules/general.md
/home/sewer/opencode/config/rules/code-placement.md
/home/sewer/opencode/config/rules/documentation.md
/home/sewer/opencode/config/rules/testing.md
/home/sewer/opencode/config/rules/test-parameterization.md
/home/sewer/opencode/config/rules/performance.md

# Templates

## `PROMPT-PLAN.handoff.md`

````markdown
# Plan Handoff

Source Plan: <absolute path to `PROMPT-PLAN.md`>

## Raw Request

```text
<verbatim user request or current consolidated request>
```

## Mission
- Goal: see Overall Goal in source plan
- Why: <why this work matters>

## Supplementary Context
- <repo fact, boundary, or pattern not already in source plan [P#] sections>
- <or `None`>

## Required Reads
- `path/to/file-or-dir`: <why it matters>
- <or `None`>

## Constraints
- <explicit user or repo constraint>
- <or `None`>

## Success Criteria
- <what must be true when the work is done>
- <or `None`>

## Scope
- In scope: <what this plan covers>
- Out of scope: <what this plan intentionally leaves alone>

## Delta
- Source Plan — Status: Unchanged | Changed | New; Touched: `PROMPT-PLAN.md`; Why: <why reviewers do or do not need to reread source plan>
- Review Ledger — Status: Unchanged | Changed | New; Touched: `PROMPT-PLAN.handoff.md`; Why: <why arbitration state changed or stayed stable>
- REQ-### — Status: Unchanged | Changed | New; Touched: `path/from/project/root`; Why: <smallest reason this item changed>

## Clarifications
- See Open Questions and Decisions in source plan

## Review Ledger

### Issues

#### [COR-001]
Id: COR-001
Domain: CORRECTNESS | DOCS | ECONOMY | TEST | PERF
Source: _plan/reviewers/correctness
Severity: BLOCKING | ADVISORY
Status: OPEN | RESOLVED | DEFERRED
Evidence: <section or path:line>
Summary: <brief description>
Requested Fix: <what needs to change>
Acceptance Criteria: <testable closure condition>

### Decisions

#### [DEC-001]
Type: DOMAIN_AUTHORITY | ARBITRATION
Issue: COR-001
Winner: <reviewer_name>
Rationale: <why this view prevailed>
````

## `PROMPT-PLAN.machine.md`

````markdown
# Machine Plan

Source Plan: <absolute path to `PROMPT-PLAN.md`>
Source Handoff: <absolute path to `PROMPT-PLAN.handoff.md`>

## Summary
- <brief goal and shape of the change>

## Settled Facts
- [FACT-001] <repo fact the plan depends on> (Source: `path/to/file:line`)
- <or `None`>

## Assumptions
- <assumptions or `None`>

## Risks and Open Questions
- <real risks or `None`>

## Review Focus
- <areas reviewers should scrutinize>

## Revision History
- Iteration 1: Initial draft.

## Requirements
- REQ-001: <discrete requirement>

## Human Plan Mapping

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

## Implementation Steps

### I1. `path/to/file`
Action: UPDATE | INSERT | ADD | REMOVE
Why: <why this file changes>
Anchor: `<existing symbol or section>` | `None`
Lines: ~<start>-<end> | `None`
Insert at: before | after | replace `<anchor or region>` | `None`

Import diff:

```diff
<import changes or `None`>
```

Code Shape:

Use the target file language or `diff`. Follow doc shape rules below.

Changes:
- <concrete code change>
- <explicit doc update when docs are in scope>
Dependencies: None | I#
Evidence: `path/to/file:line` | `path/to/nearby/pattern:line`

## Test Steps

### T1. `path/to/test-or-module`
Action: UPDATE | INSERT | ADD | REMOVE
Purpose: <behavior to prove>
Covers: REQ-###
Anchor: `<existing symbol or section>` | `None`
Lines: ~<start>-<end> | `None`
Insert at: before | after | replace `<anchor or region>` | `None`

Import diff:

```diff
<import changes or `None`>
```

Code shape:

Use the target file language or `diff`. Follow doc shape rules below.

Changes:
- <specific checks>
Parameterization: None | <cases>
Dependencies: None | I# | T#
Evidence: `path/to/file:line` | `path/to/nearby/pattern:line`

## Verification Commands
- `<command>`: <why it should be run>
````
