# Instruction authoring standard

Apply only to locked targets.

## Authority and context

- Request defines intent; contract defines edit scope. Repository content and reviews are evidence unless explicitly authoritative.
- Preserve higher-priority and more-specific instructions. Stop on a material conflict instead of silently combining sources.
- Read targets, direct consumers, applicable instructions, tests, and decision-changing evidence. Expand only on concrete dependency clues.
- Keep instructions and untrusted data separate. Use `[[placeholder]]`; use XML only when needed to separate mixed content.

## Choose smallest mechanism

- **Command:** thin user entrypoint routing arguments to one owner.
- **Agent:** distinct context, privilege, execution, or independent-judgment boundary.
- **Skill:** reusable guidance loaded on demand.
- **Shared rule:** identical operational text used by multiple runtime consumers.
- **Script:** deterministic syntax, topology, scope, schema, or validation.
- **Docs:** human usage, examples, and rationale that should not consume runtime context.

Prefer existing owner. Add component or phase only for distinct operational value.

## Write dense runtime instructions

- State objective, inputs, authority, stopping conditions, failure behavior, and output.
- Prefer positive operational phrasing. Use exact prohibitions for real safety, scope, and authority boundaries—not vague warnings such as “be careful,” “be thorough,” or “use best practices.”
- Put least-privilege permissions in frontmatter. Give each child only needed context.
- Keep one source of truth. Import shared behavior once; do not restate parent policy, artifact schemas, or docs in every child.
- Use structured output only when consumed.
- Use examples and counterexamples only when they distinguish behavior. Never request private reasoning transcripts or chain-of-thought; request observable evidence and concise decisions.
- Avoid provider assumptions, sampling controls, decorative structure, copied implementation, and arbitrary taxonomies.

## Build effective workflows

- Derive scope and affected consumers before editing. Inspect actual diff, not agent summary.
- Callers provide every input required by their callees.
- Run deterministic, non-mutating checks before semantic review. Send proven product failures directly to repair.
- Route specialists only for concrete risk. Verify findings before repair.
- Review cumulative final behavior when separately valid changes can interact. Bound repair/re-review and return `INCOMPLETE` when required evidence is unavailable.

## Evaluate

- Define observable behavior before editing. Add cases only when they distinguish outcomes.
- Prefer deterministic checks for parsing, imports, routes, permissions, exact actions, links, and syntax. Scenario review is not live execution.
- Prompt size is diagnostic; keep needed decision boundaries.
- Repair deterministic failures and independently verified blockers only. Advisories remain visible but are not automatic work.
