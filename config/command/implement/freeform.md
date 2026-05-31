---
description: "Materialize chat context into a finalized plan, then implement it"
agent: _implement/freeform
---

Use existing chat as the request. Edge case: this command body is the runtime workflow because `/implement/freeform` runs with prior chat context; do not move these instructions into `_implement/freeform.md`.

# User Arguments

```
$ARGUMENTS
```

# Inputs
- Existing conversation context is the request source; User Arguments may refine it.
- Derive `slug` from request context as a 2–3 word identifier and `artifact_base = PROMPT-PLAN-<slug>`.
- Use `plan_path = <cwd>/<artifact_base>.draft.md`, `handoff_path = <cwd>/<artifact_base>.handoff.md`, `step_pattern = <cwd>/<artifact_base>.step.*.md`, and `implementation_cache_path = <cwd>/artifact/<artifact_base>.review-implementation.md`.

# Workflow

## 1. Preflight
- Stop with `FAIL` when no implementable request or prior plan exists.
- Stop with `FAIL` for unsafe targets: `*.env`, `*.env.*`; allow `*.env.example`.
- Treat this command invocation as the confirmation boundary. Do not ask for approval.

## 2. Draft
- Write only `plan_path` from prior chat, latest user message, and User Arguments.
- Include raw request, constraints, acceptance checks, target hints, and ordered implementation/test/doc work.
- Treat the draft as confirmed for chained finalize. Do not rewrite unrelated `PROMPT-PLAN-*` artifacts.

## 3. Chained finalize
- Dispatch `_plan/finalize` with `plan_path`, `artifact_base`, and short user notes only.
- Require `Status: SUCCESS` before implementation.
- Resolve exact step paths from the returned `handoff_path` Step Index; do not infer from stale globs when the handoff lists files.

## 4. Implement
- Dispatch `_implement/freeform-implementer` with `plan_path`, `handoff_path`, exact `step_paths`, validation expectations from the handoff, `reviewer_findings: None`, and short notes.
- Wait for the implementer result; do not treat implementer success as final verification.

## 5. Review
- Dispatch `_implement/freeform-reviewer` with `plan_path`, `handoff_path`, exact `step_paths`, implementer changed paths, validation output, `cache_path: implementation_cache_path`, and optional `actions_path`.
- Accept only a fenced `# REVIEW` pointer containing `Cache:`, `Actions:`, `Agent: _implement/freeform-reviewer`, and `Decision: PASS | ADVISORY | BLOCKING`.
- Read `actions_path` for current findings. Do not read reviewer cache for fixes.
- If BLOCKING: rerun `_implement/freeform-implementer` with `reviewer_findings` from `actions_path`, affected step paths, and touched validation expectations; then re-review with the same cache path.
- If fixes invalidate plan assumptions, rerun `_plan/finalize` before the next implementer pass.
- Stop after 5 implementation-review iterations. Success requires zero unresolved BLOCKING findings.

# Output
Return exactly one fenced `text` block:

```text
Status: SUCCESS | INCOMPLETE | FAIL
Plan Path: <absolute path | N/A>
Handoff Path: <absolute path | N/A>
Step Pattern: <absolute glob | N/A>
Review Iterations: <n>
Files Changed: <comma-separated paths | None>
Summary: <one-line summary>
```
