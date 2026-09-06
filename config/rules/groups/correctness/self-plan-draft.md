## Plan Draft

### Fidelity and acceptance
Trace every user requirement/constraint across the bundle.
Use acceptance, decisions, invariants, or explicit non-goals.

Acceptance needs observable behavior, stable contracts, or executable checks.
File lists or internal steps are not acceptance.

Cohorts advance the goal within approved decisions and exclusions.

For investigation-only requests, plan discovery, not implementation.
Require acyclic dependencies and valid intermediate repository states.

### Risk and readiness
Route `CORRECTNESS` and `QUALITY` always.
Require quality before every implementation commit.
Route `PERFORMANCE` always:
only docs-only cohorts may record `NO` with a reason.

Route `TESTS` for changed observable behavior; justify inapplicable tests.

Concrete trust, auth, secrets, or IPC risks require `SECURITY`.
Concrete untrusted-input or filesystem/shell/SQL risks require `SECURITY`.
Concrete serialization, crypto, permissions, or dependency risks do too.

Acceptance or verification must reflect material risks:
security, migration, compatibility, documentation, and workload scale.

Check changed/replaced/removed behavior that comments or docs might reference.
Unresolved public-API compatibility obligations there are blocking questions.

Invent no readiness answers, old-behavior comments, or external API contracts.

Reject speculative patch mechanics prone to staleness or undue constraints.

{{ file="./rules/cards/structure/plan-bundle.md" }}

### Human entry point
- Start with `# [[title]]` and `Status: DRAFT | READY_FOR_IMPLEMENT`.
- Add a one-line `Source Request`.
- Begin with:
1. `## Start here`: reading order linking contract and cohort index.
2. `## Cohorts`: ordered links with short names and one outcome each.
   - Use IDs like `01`, `02` with dependencies or `None`.
3. `## Milestones`: partial capabilities with cohort references, not completion.
4. `## Open Questions`: decision, `Blocking: YES | NO`, affected checks/cohorts.
   - Use `None` if empty.
5. `## Final validation`: parent-owned full commands and final routes only here.
- Blocking questions block bundle readiness.

### Shared contract
- Record goal, scope/exclusions, decisions, and invariants once.
- Keep shared acceptance/review policy here, with rationale only for ambiguity.
- Each acceptance obligation names its cohort owner and observable evidence.
- Reference source sections/checks without duplicate inventories or matrices.
- Append IDs without renumbering; preserve decision and cohort/check aliases.

### Cohort document
- Front-load `## Goal`, `## Scope`, `## Not in Scope`, then `## Done when`.
- Include tests/docs in scope and observable acceptance with a stop point.
- Use reasoned `None` for inapplicable tests/docs.
- Follow with prerequisites, check references, context, targets, and impact.
- Include symbol anchors and governing instruction paths with reasons.
- Add repository-grounded validation and risk-based review routes.
- Verify paths/symbols or mark `new` targets under plausible existing modules.
- Bound placement discovery to implementation details, not open design.
- Use `None found` for ungrounded commands.
- Use anchors or short interface/data shapes only to clarify approved decisions.
- Omit exact line ranges, diffs, and near-final bodies.
