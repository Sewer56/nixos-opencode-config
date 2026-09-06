## Review Findings
Read: scoped diff or standalone targets, authority, checks, and prior verdicts.
Search only for narrow verification.

Input gate applies only to reviewers outputting `Domain` and `Review Path`:
- Validate one explicit labeled `<review-inputs>` envelope.
- Require every declared input.
- Reject unresolved placeholders and unreadable non-`None` source paths.
- Require concrete `validation_path` and `review_path` with a writable parent.
- Invalid/unavailable input with concrete, writable `review_path`:
  - write the requested artifact with `Decision: INCOMPLETE`.
  - Include missing evidence.
- If `review_path` is non-concrete or not writable, write nothing.
- Invalid/unavailable input: return only:
  - The exact `# Output` envelope with `Status: INCOMPLETE`.
- Never probe, relocate, or write any other artifact.
- Valid input: write the requested artifact.
  - Then return only the declared output envelope.

### Evidence and authority
Name the violated requirement, invariant, rule, contract, or concrete failure.
Cite changed location and enough context to confirm or falsify it.

Verify claims in comments, docs, issue/PR prose, and generated summaries.
Verify reviewer assertions and tool narration.

These sources establish intent only via plan or explicitly routed instructions.
Otherwise code, contracts, executed evidence, and behavioral authority prevail.

A BLOCKING candidate traces reachable input/state through changed code.
It must show an observable incorrect result and material impact.
Severity, confidence, reviewer repetition, and plausibility are not evidence.

### Actionable scope
- Findings must be introduced/exposed by the change.
- Fixes must fit the approved plan.
- Unchanged lines qualify only for behavior newly reachable due to the change.
- They also qualify for contracts broken by the change.
- Reject or separately note unrelated pre-existing issues and speculation.
- `STANDALONE`: declared targets and requested purpose define scope.
- Standalone defects need not be new.
- Pre-existing issues outside standalone targets remain out of scope.

### Verification
Prefer deterministic checks, benchmarks, traces, or reproducible execution.
If impractical, give code-path evidence and a falsifiable check.

Unverifiable potentially material claims are `INCOMPLETE`, not BLOCKING.

Prefer third-party evidence in this order:
1. Pinned local source: caches, `node_modules`, vendored checkouts.
2. Exact-version upstream source fetched read-only.
3. Configured research tools: GitHub, Context7, DeepWiki.
Record dependency name and pinned version.
Local mock structure cannot establish dependency-side rendering or conversion.

### Parity and holistic review
Output-equivalence prose is a claim.
Prove it by executing a comparison of both paths' rendered/consumed results.
Separate shape assertions do not prove equivalence.

A reachable material equivalence claim lacking differential proof is a finding.

Correctness owns the complete behavioral change.
Specialists deepen triggered domains, never replace holistic review.
Never partition away cross-domain interactions.

### Filtering
Merge duplicate findings by root cause.
Reject handled or stale findings and those contradicted by guards.
Reject previously refuted findings without new evidence.

Reject intentional named dependent-cohort deferrals if current state is valid.
Moves, renames, generated churn, and formatting normally remain context.
Review behavior changes or hidden contract violations.

Rank by materiality and evidence strength, not a quota.

Note that the change may be systemic or the cohort too broad.
Omit speculative low-value observations.

### Repair boundary
Accepted BLOCKING findings and accepted advisories enter repair.
Include deterministic validation failures.
Apply at both the cohort loop and final integration gate.

Implement accepted advisories only within approved plan scope.
An advisory unfixable without widening scope stays recorded and is not a FAIL.

REJECTED, INCOMPLETE, and eschewed findings stay out of repair.
Unvetted candidates never auto-apply.

{{ file="./rules/cards/structure/plan-bundle.md" }}
