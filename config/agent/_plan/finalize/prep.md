---
mode: primary
description: Resolves draft, validates preconditions, dispatches explorer, dispatches code generation, and writes pipeline state
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*": deny
    "*PROMPT-PLAN*.pipeline-state*.md": allow
  glob: allow
  list: allow
  todowrite: allow
  external_directory: allow
  task:
    "*": deny
    "_plan/finalize/explorer": allow
    "_plan/finalize/code-generate": allow
---

Resolve the confirmed draft plan, validate preconditions, dispatch repo discovery, dispatch code generation, and write a pipeline state file.

# Inputs
- The latest user message may name an exact `PROMPT-PLAN-*.draft.md` path, imply a slug, or provide finalize-time notes.
- Derive `slug` from request context as a 2–3 word identifier. Derive `artifact_base` as `PROMPT-PLAN-<slug>`.

# Artifacts
- `artifact_base`: `PROMPT-PLAN-<slug>`
- `plan_path`: `<artifact_base>.draft.md`
- `state_path`: `<artifact_base>.pipeline-state.md`
- `handoff_path`: `<artifact_base>.handoff.md`
- `discovery_path`: `artifact/<artifact_base>.repo-discovery.md`
- `step_pattern`: `<artifact_base>.step.*.md`

# Process

## 1. Resolve draft path
- If latest user message names exact `PROMPT-PLAN-*.draft.md` path, use it directly. Skip glob.
- Else if slug is clear from context, derive `artifact_base` as `PROMPT-PLAN-<slug>` and use `<artifact_base>.draft.md`. Skip glob.
- Else run exactly ONE glob for `PROMPT-PLAN-*.draft.md` in current workspace.
  - Exactly one match → proceed.
  - Zero matches → return `Status: FAIL`, write minimal state file, stop.
  - Multiple matches → return `Status: FAIL` with "multiple drafts" reason, write minimal state file, stop.

## 2. Validate draft
- Derive `artifact_base` from resolved path.
- Read `plan_path`. If read fails or file is missing, return `Status: FAIL`, write minimal state file, stop.
- Confirm the file is a well-formed draft plan. If structurally invalid, return `Status: FAIL`.

## 3. Dispatch explorer
- Dispatch `_plan/finalize/explorer` with `plan_path` and `discovery_path`.
- Record explorer status and any gaps.

## 4. Dispatch code generation
- Dispatch `_plan/finalize/code-generate` with `plan_path`, `discovery_path`, `handoff_path`, and user notes from the latest message.
- If explorer returned FAIL or discovery is missing, still dispatch — code generation can proceed with gaps.
- On `Status: FAIL`: record the failure, write minimal state, and return `Status: FAIL`.
- On success: record step count.

## 5. Write pipeline state
- Overwrite `state_path` with the `# Pipeline State Format` below.
- Include all resolved paths, validation results, explorer outcome, and handoff outcome.

# Pipeline State Format

```markdown
# Pipeline State
Artifact Base: <artifact_base>

## Resolved Paths
- plan_path: <absolute path to draft>
- handoff_path: <absolute path to handoff>
- discovery_path: <absolute path to discovery cache>
- step_pattern: <glob pattern>

## Validation
- Draft Found: true | false
- Draft Valid: true | false

## Explorer
- Status: SUCCESS | FAIL | NOT_RUN
- Discovery Path: <discovery_path>
- Gaps: <summary or none>

## Handoff
- Status: SUCCESS | FAIL | NOT_RUN
- Handoff Path: <handoff_path>
- Step Count: <n> implementation, <m> test

## User Notes
- <finalize-time notes from user message or none>
```

# Output

Return exactly:

```text
Status: SUCCESS | FAIL
State Path: <absolute state_path>
Plan Path: <absolute plan_path>
Handoff Path: <absolute handoff_path | N/A>
Discovery Path: <absolute discovery_path>
Summary: <one-line summary>
```
