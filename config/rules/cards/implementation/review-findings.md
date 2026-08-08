### Evidence first
A finding identifies a violated requirement, invariant, repository rule, contract, or concrete failure mode. It cites the changed location and enough surrounding behavior to falsify or confirm the claim.

### Claims are not proof
Treat comments, documentation, issue or PR prose, generated summaries, reviewer assertions, and tool narration as claims to verify. They may establish approved intent only when represented by the plan or an explicitly routed instruction source; otherwise they cannot override code, contracts, executed evidence, or the approved behavioral authority.

### Reachable failure and impact
A BLOCKING candidate states a reachable path from input/state through the changed code to an observable incorrect result, plus the material impact. Severity labels, confidence scores, reviewer repetition, and generic plausibility are not evidence.

### Actionable scope
The problem must be introduced or exposed by the reviewed change and fixable within the approved plan. An unchanged line may be cited only when the change makes its behavior newly reachable or breaks its contract. Pre-existing unrelated issues and speculative future improvements are rejected or noted separately.

For `STANDALONE` review of declared current targets, the requested target and purpose define scope; a defect need not be newly introduced. Pre-existing issues outside declared targets remain out of scope.

### Verification
Prefer compiler, type checker, test, linter, static-analysis, benchmark, trace, or reproducible execution evidence. Reasoned code-path evidence is acceptable when deterministic proof is impractical, but must state a falsifiable check. A potentially material claim that cannot be verified with the available evidence or environment is `INCOMPLETE`, not BLOCKING.

### Holistic first, specialists second
Correctness review remains responsible for the complete behavioral change. Specialist reviewers add depth for triggered risk domains; they do not partition away cross-domain interactions or replace the holistic pass.

### Deduplication, drift, and mechanical noise
Merge duplicate findings by root cause. Reject findings already handled in the current tree, based on stale line context, contradicted by an existing guard, intentionally deferred to a named dependent cohort without invalidating the current state, or previously refuted without new evidence. Treat pure moves, renames, generated churn, and formatter-only edits as context rather than semantic defects unless they change behavior or hide a contract violation.

### Repair boundary
Accepted BLOCKING findings and accepted advisories enter repair in the cohort loop and at the final integration gate alike, alongside deterministic validation failures. Implement an accepted advisory only within approved plan scope; one that cannot be fixed without widening scope stays recorded and is not a FAIL. REJECTED, INCOMPLETE, and eschewed findings stay out of repair; unvetted reviewer candidates never auto-apply.

### Signal budget
Normally emit no more than five findings for one small cohort and one domain, ordered by materiality and evidence strength. Do not fill a quota. If more than five independent material blockers survive, emit every blocker, suppress advisories, and note that the change may be systemic or the cohort may be too broad. Omit speculative low-value observations.
