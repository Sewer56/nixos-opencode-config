### Evidence first
A finding identifies a violated requirement, invariant, repository rule, contract, or concrete failure mode. It cites the changed location and enough surrounding behavior to falsify or confirm the claim.

### Claims are not proof
Treat comments, documentation, issue or PR prose, generated summaries, reviewer assertions, and tool narration as claims to verify.

They establish approved intent only when represented by the plan or an explicitly routed instruction source; otherwise they cannot override code, contracts, executed evidence, or behavioral authority.

### Reachable failure and impact
A BLOCKING candidate states a reachable path from input/state through the changed code to an observable incorrect result, plus the material impact.

Severity labels, confidence scores, reviewer repetition, and generic plausibility are not evidence.

### Actionable scope
- **Change-introduced, plan-fixable**: The problem must be introduced or exposed by the reviewed change and fixable within the approved plan.
- **Unchanged-line citations**: An unchanged line may be cited only when the change makes its behavior newly reachable or breaks its contract.
- **Reject or separate the unrelated**: Pre-existing unrelated issues and speculative future improvements are rejected or noted separately.
- **`STANDALONE` scope**: For `STANDALONE` review of declared current targets, the requested target and purpose define scope; a defect need not be newly introduced. Pre-existing issues outside declared targets remain out of scope.

### Verification
Prefer compiler, type checker, test, linter, static-analysis, benchmark, trace, or reproducible execution evidence.

Reasoned code-path evidence is acceptable when deterministic proof is impractical, but must state a falsifiable check.

A potentially material claim that cannot be verified with the available evidence or environment is `INCOMPLETE`, not BLOCKING.

### External dependency evidence
For claims that depend on third-party behavior, acceptable evidence is, best-first:

- The pinned dependency source already on disk (cargo registry cache, `node_modules`, vendored checkouts).
- The exact-version upstream source fetched read-only.
- Configured research tools (GitHub, Context7, DeepWiki).

Record the dependency name and pinned version.

Structural assertions on a local mock cannot establish dependency-side rendering or conversion behavior.

### Parity claims need differential evidence
Prose claiming two paths produce equivalent output ("byte-identical", "same as X") is a claim.

Equivalence is proven only by an executed comparison of both paths' rendered or consumed results; separate per-path shape assertions do not satisfy it.

A reachable, material equivalence claim without differential evidence is a finding.

### Holistic first, specialists second
Correctness review remains responsible for the complete behavioral change. Specialist reviewers add depth for triggered risk domains; they do not partition away cross-domain interactions or replace the holistic pass.

### Deduplication, drift, and mechanical noise
Merge duplicate findings by root cause.

Reject findings already handled in the current tree, based on stale line context, contradicted by an existing guard, or previously refuted without new evidence.

Reject findings intentionally deferred to a named dependent cohort without invalidating the current state.

Treat pure moves, renames, generated churn, and formatter-only edits as context rather than semantic defects unless they change behavior or hide a contract violation.

### Repair boundary
Accepted BLOCKING findings and accepted advisories enter repair in the cohort loop and at the final integration gate alike, alongside deterministic validation failures.

Implement an accepted advisory only within approved plan scope; one that cannot be fixed without widening scope stays recorded and is not a FAIL.

REJECTED, INCOMPLETE, and eschewed findings stay out of repair; unvetted reviewer candidates never auto-apply.

### Signal budget
Normally emit no more than five findings for one small cohort and one domain, ordered by materiality and evidence strength; do not fill a quota.

If more than five independent material blockers survive, emit every blocker, suppress advisories, and note that the change may be systemic or the cohort may be too broad.

Omit speculative low-value observations.
