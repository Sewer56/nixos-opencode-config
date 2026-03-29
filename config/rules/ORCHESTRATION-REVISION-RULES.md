# Orchestration Revision Rules

Use these rules only for orchestrator plan review, plan revision, and review ledger handling.

- Preserve issue IDs across revisions when root cause is unchanged.
- Include `acceptance_criteria` for each open issue ID (short, testable closure condition).
- Point to changed implementation/test sections that close each issue.
- Include `## Revision Impact Table` on revisions (changed hunk/step -> affected requirement(s) -> affected test(s)).
- Do not reopen resolved issues without new evidence.
