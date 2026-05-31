---
mode: primary
description: Runs finalize, code-doc finalize, and user-doc finalize as one chained plan-finalize workflow
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  glob: allow
  grep: allow
  list: allow
  external_directory: allow
  task:
    "*": "deny"
    "_plan/finalize": "allow"
    "_plan/finalize-code-docs": "allow"
    "_plan/finalize-user-docs": "allow"
---

Run the three existing finalize workflows as one chain. Preserve the same artifact contract as running `/plan/finalize`, `/plan/finalize-code-docs`, then `/plan/finalize-user-docs` manually.

# Inputs
- The latest user message may name an exact `PROMPT-PLAN-*.draft.md` path, a slug, or finalize-time notes.
- Derive `slug` from request context as a 2–3 word identifier and `artifact_base = PROMPT-PLAN-<slug>`.
- Use `plan_path = <cwd>/<artifact_base>.draft.md`, `handoff_path = <cwd>/<artifact_base>.handoff.md`, `step_pattern = <cwd>/<artifact_base>.step.*.md`, and shared cache `<cwd>/artifact/<artifact_base>.repo-discovery.md`.

# Workflow

## 1. Resolve draft
- If the user names an exact draft path, use it directly.
- Else if a slug is clear, derive `plan_path` from `artifact_base`.
- Else run one targeted glob for `PROMPT-PLAN-*.draft.md`; stop with `FAIL` if zero or multiple drafts match.
- Do not read broad repo context before `plan_path` is resolved.

## 2. Finalize code and tests
- Dispatch `_plan/finalize` with `plan_path` and compact finalize-time notes.
- Stop if it returns `Status: FAIL` or unresolved BLOCKING findings.
- Use its returned `handoff_path` and `step_pattern` as the shared context for later stages.

## 3. Finalize code-adjacent docs
- Dispatch `_plan/finalize-code-docs` with `plan_path`, `handoff_path`, `step_pattern`, and compact notes.
- Stop if it returns `Status: FAIL` or unresolved BLOCKING findings.
- Keep code-doc findings and caches owned by that child workflow.

## 4. Finalize user docs
- Dispatch `_plan/finalize-user-docs` with `plan_path`, `handoff_path`, `step_pattern`, and compact notes.
- If the child determines no user-facing documentation work applies, accept its `Status: SUCCESS` and keep the shared handoff as the ledger.
- Stop if it returns `Status: FAIL` or unresolved BLOCKING findings.

## 5. Finish
- Read only returned status blocks and, when needed, the shared `handoff_path` Step Index to confirm current step files.
- Let child finalize workflows own artifact writes and review loops.

# Output
Return exactly one fenced `text` block:

```text
Status: SUCCESS | INCOMPLETE | FAIL
Plan Path: <absolute path to `<artifact_base>.draft.md` | N/A>
Handoff Path: <absolute path to `<artifact_base>.handoff.md` | N/A>
Step Pattern: <absolute glob | N/A>
Review Iterations: <n>
Summary: <one-line summary>
```

# Constraints
- Pass children only paths, trigger flags, and short notes.
- Child workflows use the shared discovery cache and targeted local reads for named gaps.
- Call only the three finalize parent agents.
- Preserve child cache/action ownership. Use the handoff as the shared ledger.
- Leave product code and git history unchanged.
