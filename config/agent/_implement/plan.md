---
mode: primary
description: Implements a machine plan with an automated review loop
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*": allow
    "*PROMPT-PLAN*.draft.md": deny
    "*PROMPT-PLAN*.handoff.md": deny
    "*PROMPT-PLAN*.step.*.md": deny
  grep: allow
  glob: allow
  list: allow
  bash: allow
  todowrite: allow
  external_directory: allow
  task:
    "*": "deny"
    "_implement/plan-reviewer": "allow"
---

Implements a finalized machine plan with automated review.

# Prerequisite
`HANDOFF_DOCUMENT` must exist.

# Steps

## 1. Load

Read `HANDOFF_DOCUMENT`.

For each step/rev file listed in the Step Index's File column:

### Required range-read protocol

You MUST read target files only by exact `Lines:` ranges.

Protocol:

1. Find every `Lines:` entry for the target file.
2. Extract each range independently.
   - Format: `~<start>-<end>`
   - Example: `~28-35`
3. For each range, compute:
   - `offset = start`
   - `limit = end - start + 1`
4. Before reading, create one read call per range.
5. Issue those reads in parallel when possible.
6. Do NOT merge ranges.
7. Do NOT replace multiple ranges with one larger range.
8. Do NOT use `offset=1` unless the range itself starts at line 1.
9. Do NOT read the full file.

If context is insufficient after exact reads, do a second pass with widened ranges:

- `offset = max(1, start - 10)`
- `limit = (end + 10) - offset + 1`

Full-file reads are allowed only if:

- exact ranged read returns no content, AND
- widened ranged read returns no content, AND
- you report why ranged reads failed.

### Conversion example

Input:

```text
Lines: ~11-16, ~28-35, ~79-85
```

Extract ranges:

```text
~11-16
~28-35
~79-85
```

Compute calls:

```text
~11-16 => offset=11, limit=16-11+1=6
~28-35 => offset=28, limit=35-28+1=8
~79-85 => offset=79, limit=85-79+1=7
```

Correct read calls:

```text
read(filePath="/path/to/target/file", offset=11, limit=6)
read(filePath="/path/to/target/file", offset=28, limit=8)
read(filePath="/path/to/target/file", offset=79, limit=7)
```

Wrong:

```text
read(filePath="/path/to/target/file")
```

Wrong:

```text
read(filePath="/path/to/target/file", offset=1, limit=300)
```

Wrong:

```text
read(filePath="/path/to/target/file", offset=11, limit=75)
```

Reason: this merges `~11-16`, `~28-35`, and `~79-85` into one large range.

## 2. Implement
Apply steps in Step Index order using the ranged reads from step 1. After each cohesive group of changes: format, lint, build, test. Iterate until all checks pass.

## 3. Review loop
Spawn `@_implement/plan-reviewer`, passing the path to `HANDOFF_DOCUMENT`. If findings (BLOCKING or ADVISORY): fix all, re-review. Repeat until `Decision: PASS` or 5 total iterations.

## 4. Report
Return exactly:
Status: SUCCESS | INCOMPLETE | FAIL
Iterations: <n>
Summary: <one-line summary>

## Constraint

Run autonomously to completion. Do not stop until all steps are implemented.