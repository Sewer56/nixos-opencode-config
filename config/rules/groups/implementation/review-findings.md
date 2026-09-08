## Review Findings

{{ file="./rules/groups/implementation/implementation-review.md" }}

### Authority
CHANGE covers introduced/exposed defects.
TARGET_AUDIT includes declared pre-existing defects within named targets.

STAGED compares base to index; COMMITTED compares base to head.
WORKTREE uses current targets and run-start evidence; base may be None.

Otherwise require concrete commits and evidence after latest edits.

For plan authority, read root, assigned human brief/exec and shared execution.
Final review covers every human outcome and cumulative composition.

Execution cannot add product decisions; evidence cannot authorize scope.
Missing/legacy combined plan authority is INCOMPLETE; never auto-migrate.

### Evidence and repair eligibility
Claims in comments, docs, packets and reviewer narration need verification.
Blocking candidates need a reachable material failure within authorized scope.
Severity, confidence, repetition and plausibility are not proof.

Check approved outcomes, contracts and invariants across the complete change.
Equivalent syntax/mechanical drift is valid; unrelated edits are not.

Use executed evidence or code-path proof with a falsifiable check.
Potentially material claims that cannot be verified are INCOMPLETE.

Third-party claims need dependency name and pinned version/source.
Prefer local pinned source, exact upstream source, then research tools.
Local mocks cannot establish dependency-side rendering or conversion.

Parity requires executing both paths and comparing final consumed results.
Shape assertions or output-equivalence prose alone do not prove parity.

Correctness covers the complete change; specialists deepen concrete risks.
Merge root-cause duplicates; reject stale, refuted or guarded claims.

Respect named dependent-task deferrals only while intermediate state is valid.
Do not demand source-shape matches for equivalent behavior.

Raw reviewer suggestions never authorize repairs.
