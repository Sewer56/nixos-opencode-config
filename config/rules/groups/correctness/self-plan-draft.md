## Plan Draft

### Fidelity and acceptance
Trace every user requirement/constraint to an acceptance obligation, decision, invariant, or explicit non-goal across the bundle.

Acceptance needs observable behavior, stable contracts, or executable checks, not file lists or internal steps.

Map every obligation to cohort evidence without duplicate matrices.
Cohorts must advance the goal without contradicting approved decisions or exclusions.

Investigation-only requests plan discovery, not implementation.
Require acyclic dependencies and valid intermediate repository states.

Verify paths/symbols or mark plausible new targets under existing modules.
Bounded placement discovery is for implementation details, not unresolved design.

### Risk and readiness
Route `CORRECTNESS` and `QUALITY` always, with quality before every implementation commit.
Route `PERFORMANCE` always; only docs-only cohorts may record `NO` with a reason.

Route `TESTS` for changed observable behavior; justify inapplicable tests.

Route `SECURITY` for concrete trust, auth, secrets, IPC, untrusted input, filesystem/shell/SQL, serialization, crypto, permissions, or dependency risks.

Reflect material security, migration, compatibility, documentation, and workload-scale risks in acceptance or verification.

For changed/replaced/removed behavior comments or docs might reference, unresolved public-API compatibility obligations are blocking questions.

Do not invent an old-behavior comment, external API contract, or answer to establish readiness.
Reject speculative patch mechanics that falsely constrain valid implementation or become stale.

{{ file="./rules/cards/structure/plan-bundle.md" }}

### Human entry point
- Start with `# [[title]]` and `Status: DRAFT | READY_FOR_IMPLEMENT`.
- Add a one-line `Source Request`.
- Begin with:
1. `## Start here`: reading order linking contract and cohort index.
2. `## Cohorts`: ordered links with short names and one outcome each.
   - Use IDs like `01`, `02`; give dependencies or `None`.
3. `## Milestones`: partial capabilities with cohort references, not completion.
4. `## Open Questions`: decision, `Blocking: YES | NO`, affected checks/cohorts.
   - Use `None` if no questions remain.
5. `## Final validation`: parent-owned full commands and final routes only here.
- Blocking questions invalidate readiness for the whole bundle.

### Shared contract
- Record goal, scope/exclusions, decisions, and invariants once.
- Keep shared acceptance/review policy here.
- Include rationale only for ambiguity.
- Each acceptance obligation names an owning cohort and observable evidence.
- Reference source sections/checks, not duplicate inventories or matrices.
- Append IDs without renumbering; preserve decision and cohort/check aliases.
- IDs are plan-internal: never cite them in committed content/messages.

### Cohort document
- Front-load `## Goal`, `## Scope`, `## Not in Scope`, then `## Done when`.
- Include tests/docs in scope and observable acceptance with a stop point.
- Use `None` with a reason when tests or docs do not apply.
- Follow with prerequisites, check references, context, targets, and impact.
- Include symbol anchors and applicable instruction paths with reasons.
- Add repository-grounded validation and risk-based review routes.
- Paths must exist or be marked `new` under a plausible existing module.
- Allow bounded placement discovery.
- Use `None found` for ungrounded commands.
- Use anchors or short interface/data shapes only to clarify approved decisions.
- Omit exact line ranges, diffs, and near-final bodies.
