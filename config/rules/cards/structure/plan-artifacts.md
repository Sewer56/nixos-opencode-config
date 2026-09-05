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
