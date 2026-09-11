# Rules

## Ownership

- `review/`: cross-workflow reporting, review contracts and routing.
- `code/`: shared writer research and scope procedures.
- `plan/`: plan-bundle authority shared by planning and implementation.
- `write/`: shared writer prose-lint procedure.
- `commit-message.md`: shared by the commit agent and command.
- `plan-confirmation.md`: approval rules shared by Code and Docs.

## Composition

- Agents and rules import only the shared instructions they need.
- Put agent imports in one final `# Rules` section, after process and output.
- Inline a module with one direct caller into that caller.
- Keep workflow-local fragments in the nearest owning `agent/**/shared/`.
  Include the workflow's parent entry agent when determining ownership.
- Use `.txt` there; OpenCode discovers nested `.md` files as agents.

### Shared modules

- Keep cross-workflow modules here; check direct and transitive consumers.
- Merge modules always consumed together within one role.
- Recount callers after merging.
- Inline domain rules in each agent; keep only genuinely shared protocols here.
- Shared procedures and interfaces may compose other shared protocols.

### Role boundaries

- Keep role-specific instructions in the agent.
- Writers get requirements, non-obvious conventions and production procedures.
- Reviewers get audience, relevance, evidence and severity checks.
- Do not copy review checklists into writer rules for symmetry.
- Share only constraints or interfaces both roles need explicitly.
- Use matching domain/subsection headings for writer/reviewer comparisons.
- Do not add writer counterparts for reviewer-only checks.
- Duplicate a concern deliberately when its role-specific wording differs.
- Verify gate coverage before removing mechanical review checks.

## Review imports

- Candidate reviewers import `_review/shared/candidates.txt` for their return.
- Candidates and the verifier share `_review/shared/review-rules.txt`.
  It includes evidence-only writes and imports `review/contract.md` here.
- `_review/verifier` tests candidate justifications without domain checklists.
- Only `doc-quality` carries the documentation acceptance checklist, inline.
- Correctness owns test adequacy; code quality owns test organization/style.
- Writers carry inline production rules, not acceptance checklists.
- `code/writing.md` contains only writer procedures.

### Orchestrators and reporting

- Orchestrators import `review/routing.md`, not reviewer write restrictions.
- Routing owns accepted-repair procedures.
- Review rules and routing share `review/contract.md` for inputs.
- The contract imports `review/reporting.md` for evidence and report handoffs.
  Plan, writing-adherence and CodeRabbit roles also import reporting directly.

## Validation

- Compare expanded consumers when moving, merging or inlining rules.
- Preserve instructions, import arguments and role boundaries.
- The config validator checks imports, cycles and rule reachability.
- It also checks local fragment reachability and rejects `.md` in `shared/`.
- Delete obsolete groups/cards or modules after checking all consumers.
- Plugin fixtures and unrelated modules are not runtime cleanup targets.
