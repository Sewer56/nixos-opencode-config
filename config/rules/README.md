# Rules

## Ownership

- `review/`: cross-workflow reporting, review contracts and routing.
- `code/`: code-writing guidance and criteria shared with reviewers.
- `docs/`: documentation rules shared by writers and reviewers.
- `plan/`: plan-bundle authority shared by planning and implementation.
- `write/`: wording, formatting and prose lint shared across workflows.
- `commit-message.md`: shared by the commit agent and command.
- `plan-confirmation.md`: approval rules shared by Code and Docs.

## Composition

- Agents and rules import only the shared instructions they need.
- Put agent imports in one final `# Rules` section, after process and output.
- Inline a module with one direct caller into that caller.
- Keep workflow-local fragments in the nearest owning `agent/**/shared/`.
  Include the workflow's parent entry agent when determining ownership.
- Use `.txt` there; OpenCode discovers nested `.md` files as agents.
- Keep cross-workflow modules here; check direct and transitive consumers.
- Merge modules always consumed together; recount callers after merging.
- Keep role-specific instructions in the agent.

## Review imports

- Candidate reviewers import `_review/shared/review-rules.txt`.
  It includes evidence-only writes and imports `review/contract.md` here.
- Verifiers import `_review/verifiers/shared/verification.txt`.
  It imports review rules and replaces the candidate return with verdict output.
- Documentation review and quality verification share `_review/shared/docs.txt`.
- Orchestrators import `review/routing.md`, not reviewer write restrictions.
- Review rules and routing share `review/contract.md` for inputs and repairs.
- The contract imports `review/reporting.md` for evidence and report handoffs.
  Plan, writing-adherence and CodeRabbit roles also import reporting directly.

## Validation

- Compare expanded consumers when moving, merging or inlining rules.
- Preserve instructions, import arguments and role boundaries.
- The config validator checks imports, cycles and rule reachability.
- It also checks local fragment reachability and rejects `.md` in `shared/`.
