<prompt_optimization>
Scope: changed model-facing command, agent, reviewer, template, and prompt-documentation text.

<boundaries>
- Prompt text owns goal, scope, inputs, source/context boundaries, task-level tool behavior, output contract, stop/fallback rules, and verification evidence.
- Harness/config owns model choice, effort/reasoning parameters, tool schemas, MCP wiring, provider reasoning replay, permission tables, sandbox/egress, and prompt-cache mechanics.
- If editing harness/config docs, keep harness advice in those docs; do not copy it into runtime prompt bodies.
</boundaries>

<shape>
- Start with deliverable, scope, and done criteria.
- Include exact inputs, editable scope, non-goals, output schema, failure behavior, and verification evidence when they affect correctness.
- Use XML-style tags only for mixed blocks: instructions plus context, examples, source data, or schemas. Keep simple prompts flat.
- Use [[placeholder_name]] for slots. Reserve angle brackets for real XML-style tags.
</shape>

<density>
- Remove persona fluff, pleasantries, stale model workarounds, generic encouragement, copied catalogs, repeated caller/callee policy, and examples that do not constrain behavior.
- Preserve source boundaries, safety gates, schemas, permission implications, stop rules, and checks.
- Prefer positive scoped rules and if/then branches over broad always/never lists.
</density>

<context>
- Read target paths and direct references before changing repo-fact claims.
- Bound discovery by target, evidence conflict, and validation failures.
- Use subagents/reviewers only when separate context or independent judgment lowers total context cost or risk.
- Handoffs contain paths, ids, flags, criteria, and artifact paths; not parent workflows or full rule catalogs.
</context>

<verification>
- Define the smallest useful check: render, static, lint, test, build, smoke, grep, or inspectable substitute.
- Record checks not run with reason and next-best evidence.
- Treat missing or malformed machine output as a protocol failure.
</verification>

<blocking_failures>
- Missing output contract for a runtime prompt.
- Missing verification or inspectable substitute for changed behavior.
- Prompt/harness responsibility mixing.
- Placeholder/XML ambiguity.
- Untrusted source content treated as active instruction.
- Selected contract rule absent.
- Over-compression that removes a required boundary, schema, safety gate, or check.
</blocking_failures>
</prompt_optimization>
