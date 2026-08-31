### Draft identity
Draft artifacts begin with `# <descriptive title>`, then `Status: DRAFT | READY_FOR_IMPLEMENT` and a one-line `Source Request` summary.

### Required sections
Use this order:
1. `## Overall Goal`
2. `## Scope` with `### In Scope` and `### Out of Scope`
3. `## Decisions`
4. `## Constraints and Invariants`
5. `## Acceptance Criteria`
6. `## Plan`
7. `## Verification`
8. `## Risks and Notes`
9. `## Open Questions`
10. `## Relevant Files`
Use `None` for an empty required section.

### Stable identifiers
Use `[D#]` for decisions, `[INV-#]` for invariants, `[AC-#]` for acceptance criteria, `[P#]` for plan items, and `[Q#]` for questions.

Keep identifiers stable across revisions; append new ids rather than renumbering accepted content.

These identifiers are plan-internal in any form, bare or bracketed (`AC-1`, `[AC-#]`): never cite them in committed code, comments, tests, docs, or commit messages.

### Plan item shape
Each `[P#]` item has:
- `Outcome:` one coherent behavioral result.
- `Depends On:` prior `[P#]` ids or `None`.
- `Covers:` `[AC-#]` ids.
- `Targets:` verified files/symbols, plausible new targets, or a bounded area to discover during implementation.
- `Change:` contract-level behavior and explicit non-goals.
- `Tests:` observable coverage or `None` with a reason.
- `Documentation:` affected user/code docs or `None` with a reason.
- `Review Routes:` always `CORRECTNESS` and `QUALITY`; `PERFORMANCE` on every item (`NO` with a reason only when docs-only); plus grounded optional `TESTS` and `SECURITY`. `QUALITY` reviews each proposed commit before commit.
- `Completion Evidence:` checks or observations proving completion.

### Questions
Each open question states `Blocking: YES | NO`, the decision needed, and its affected `[P#]`/`[AC-#]` ids. `READY_FOR_IMPLEMENT` is invalid while a `Blocking: YES` question is open.

### Verification
Separate `### Targeted` and `### Full` commands. Commands must come from repository manifests, CI, or existing developer documentation when available. Use `None found` rather than inventing commands.

### Relevant files table
`## Relevant Files` uses columns `Path | Type | Plan Refs | Why`.

Use `change`, `contract`, `verify`, `instruction`, `test`, `docs`, `config`, or `schema` as useful type labels.

Include only files likely to change a planning or implementation decision: intended targets, direct contracts/consumers, applicable path-specific instructions, associated tests/docs, and validation owners.

A path must exist or be marked `new` with a plausible parent/module.

This table is not an exhaustive dependency inventory.

### Intent and rationale
`Source Request`, `Overall Goal`, approved decisions, invariants, acceptance criteria, and non-goals are the durable behavioral authority.

Preserve rationale only when it changes how an ambiguous implementation or review decision should be resolved; encode it with the relevant decision, invariant, risk, or non-goal rather than creating a general history transcript.

### No pseudo-patch planning
Do not require or include exact line ranges, import diffs, unified diffs, or near-final function bodies. Symbol anchors and short illustrative interface/data shapes are allowed only when they clarify an approved decision.
